use std::time::Duration;

use super::clock::Clock;
use super::payload::AllGameData;
use super::snapshot::Snapshot;

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub struct LiveClient {
    http: reqwest::Client,
    url: String,
}

impl LiveClient {
    /// # Errors
    /// If the HTTP client cannot be built, which only a broken TLS backend causes.
    pub fn new(url: &str) -> reqwest::Result<Self> {
        // The League client serves loopback HTTPS with a self-signed certificate,
        // and this client only ever talks to the fixed loopback URL.
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_millis(800))
            .build()?;
        Ok(Self {
            http,
            url: url.to_string(),
        })
    }

    /// Runs until `on_update` fails, and returns its error. `None` means no match the
    /// player is part of is running: no game, still loading, or ended.
    pub async fn poll<E>(
        &self,
        clock: &impl Clock,
        mut on_update: impl FnMut(Option<Snapshot>) -> Result<(), E> + Send,
    ) -> E {
        loop {
            if let Err(error) = on_update(self.fetch().await) {
                return error;
            }
            clock.sleep(POLL_INTERVAL).await;
        }
    }

    async fn fetch(&self) -> Option<Snapshot> {
        let response = self.http.get(&self.url).send().await.ok()?;
        let data: AllGameData = serde_json::from_str(&response.text().await.ok()?).ok()?;
        if !data.match_running() {
            return None;
        }
        let cs = data.active_player()?.scores.creep_score;
        Some(Snapshot { cs })
    }
}
