use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;

use crate::store::Store;
use crate::{Clock, CollectError, Config};

// Riot answers 403 with no rate limit headers to a request with no User-Agent of its own.
const USER_AGENT: &str = concat!("lollipop-collector/", env!("CARGO_PKG_VERSION"));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_FAILURES: u32 = 5;
const RETRY_STEP: Duration = Duration::from_secs(2);
const DEFAULT_RETRY_AFTER_S: u64 = 10;
const RETRY_AFTER_MARGIN_S: u64 = 1;

/// Every call waits for the application rate limit, which outlives a restart in `Store`.
pub(crate) struct RiotApi<'a, C> {
    http: Client,
    platform_url: &'a str,
    regional_url: &'a str,
    clock: &'a C,
}

enum Reply {
    Body(Vec<u8>),
    Missing,
    KeyRejected(u16),
    Limited,
    Failed,
}

impl<'a, C: Clock> RiotApi<'a, C> {
    pub(crate) fn new(config: &'a Config, clock: &'a C) -> Result<Self, CollectError> {
        let mut key = HeaderValue::from_str(&config.key).map_err(|_| CollectError::InvalidKey)?;
        key.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert("x-riot-token", key);
        let http = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| CollectError::Client(error.without_url()))?;
        Ok(Self {
            http,
            platform_url: &config.platform_url,
            regional_url: &config.regional_url,
            clock,
        })
    }

    pub(crate) fn now_ms(&self) -> u64 {
        self.clock.now_ms()
    }

    /// `None` is a 404.
    pub(crate) async fn platform<T: DeserializeOwned>(
        &self,
        store: &Store,
        call: &'static str,
        path: &str,
    ) -> Result<Option<T>, CollectError> {
        self.get(store, call, format!("{}{path}", self.platform_url))
            .await
    }

    /// `None` is a 404.
    pub(crate) async fn regional<T: DeserializeOwned>(
        &self,
        store: &Store,
        call: &'static str,
        path: &str,
    ) -> Result<Option<T>, CollectError> {
        self.get(store, call, format!("{}{path}", self.regional_url))
            .await
    }

    async fn get<T: DeserializeOwned>(
        &self,
        store: &Store,
        call: &'static str,
        url: String,
    ) -> Result<Option<T>, CollectError> {
        let mut failures = 0;
        loop {
            self.wait_for_slot(store).await?;
            match self.exchange(store, &url).await? {
                Reply::Body(body) => {
                    let parsed = serde_json::from_slice(&body);
                    return parsed.map(Some).map_err(|_| CollectError::Unreadable(call));
                }
                Reply::Missing => return Ok(None),
                Reply::KeyRejected(status) => return Err(CollectError::KeyRejected(status)),
                Reply::Limited => {}
                Reply::Failed if failures == MAX_FAILURES => {
                    return Err(CollectError::CallFailed(call));
                }
                Reply::Failed => {
                    failures += 1;
                    self.clock.sleep(RETRY_STEP * failures).await;
                }
            }
        }
    }

    async fn wait_for_slot(&self, store: &Store) -> Result<(), CollectError> {
        loop {
            let now = self.clock.now_ms();
            let next = store.next_call_at(now)?;
            if next <= now {
                store.record_call(now)?;
                return Ok(());
            }
            self.clock.sleep(Duration::from_millis(next - now)).await;
        }
    }

    async fn exchange(&self, store: &Store, url: &str) -> Result<Reply, CollectError> {
        let Ok(response) = self.http.get(url).send().await else {
            return Ok(Reply::Failed);
        };
        if let Some(limit) = header_text(response.headers(), "x-app-rate-limit") {
            store.learn_rate_limit(limit)?;
        }
        Ok(match response.status() {
            StatusCode::OK => response
                .bytes()
                .await
                .map_or(Reply::Failed, |body| Reply::Body(body.to_vec())),
            StatusCode::NOT_FOUND => Reply::Missing,
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Reply::KeyRejected(response.status().as_u16())
            }
            StatusCode::TOO_MANY_REQUESTS => {
                store.hold_calls_until(self.end_of_retry_after(response.headers()))?;
                Reply::Limited
            }
            _ => Reply::Failed,
        })
    }

    fn end_of_retry_after(&self, headers: &HeaderMap) -> u64 {
        let retry_after = header_text(headers, RETRY_AFTER.as_str())
            .and_then(|seconds| seconds.parse().ok())
            .unwrap_or(DEFAULT_RETRY_AFTER_S);
        self.clock.now_ms() + (retry_after + RETRY_AFTER_MARGIN_S) * 1000
    }
}

fn header_text<'h>(headers: &'h HeaderMap, name: &str) -> Option<&'h str> {
    headers.get(name)?.to_str().ok()
}
