use std::time::Duration;

use reqwest::header::RETRY_AFTER;
use reqwest::StatusCode;

use super::answer::Answer;
use super::failure::Failure;
use super::responses_request::ResponsesRequest;
use super::USER_AGENT;

/// The Responses API at a base URL, which tests point at a fake.
#[derive(Debug, Clone)]
pub(crate) struct OpenAi {
    http: reqwest::Client,
    base_url: String,
}

impl OpenAi {
    pub(super) fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// One request, with no retry.
    pub(super) async fn attempt(
        &self,
        key: &str,
        request: &ResponsesRequest<'_>,
        timeout: Duration,
    ) -> Result<Answer, Failure> {
        let response = self
            .http
            .post(format!("{}/v1/responses", self.base_url))
            .bearer_auth(key)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
            .timeout(timeout)
            .json(request)
            .send()
            .await
            .map_err(|error| failure_of(&error))?;
        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = retry_after(&response);
            let body = response.text().await.unwrap_or_default();
            return Err(if out_of_quota(&body) {
                Failure::NoQuota
            } else {
                Failure::RateLimited { retry_after }
            });
        }
        if !status.is_success() {
            return Err(Failure::Status(status.as_u16()));
        }
        Answer::read(&response.text().await.map_err(|error| failure_of(&error))?)
    }
}

fn failure_of(error: &reqwest::Error) -> Failure {
    if error.is_timeout() {
        Failure::Timeout
    } else {
        Failure::Network
    }
}

// Only the seconds form: an HTTP date reads as no header at all.
fn retry_after(response: &reqwest::Response) -> Option<Duration> {
    let seconds = response.headers().get(RETRY_AFTER)?.to_str().ok()?;
    seconds.trim().parse().ok().map(Duration::from_secs)
}

// OpenAI tells a spent quota from a rate limit only by the error code in the body.
fn out_of_quota(body: &str) -> bool {
    let Ok(body) = serde_json::from_str::<serde_json::Value>(body) else {
        return false;
    };
    ["code", "type"].iter().any(|field| {
        body.pointer(&format!("/error/{field}"))
            .is_some_and(|v| v == "insufficient_quota")
    })
}
