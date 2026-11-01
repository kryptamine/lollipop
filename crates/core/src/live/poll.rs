use std::time::Duration;

use super::clock::Clock;
use super::payload::AllGameData;
use super::snapshot::Snapshot;

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Never returns. `None` means the API has no running game the player is part of.
/// # Panics
/// If the HTTP client cannot be built, which only a broken TLS backend causes.
pub async fn poll(
    url: &str,
    clock: &impl Clock,
    mut on_update: impl FnMut(Option<Snapshot>) + Send,
) {
    // The League client serves loopback HTTPS with a self-signed certificate,
    // and this client only ever talks to the fixed loopback URL.
    #[allow(
        clippy::expect_used,
        reason = "no game data can be read without a client"
    )]
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_millis(800))
        .build()
        .expect("HTTP client should build");

    loop {
        on_update(fetch(&client, url).await);
        clock.sleep(POLL_INTERVAL).await;
    }
}

async fn fetch(client: &reqwest::Client, url: &str) -> Option<Snapshot> {
    let body = client.get(url).send().await.ok()?.text().await.ok()?;
    let data: AllGameData = serde_json::from_str(&body).ok()?;
    let cs = data.active_player()?.scores.creep_score;
    Some(Snapshot { cs })
}
