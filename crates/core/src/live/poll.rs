use std::time::Duration;

use super::clock::Clock;
use super::gold_earned::GoldEarned;
use super::payload::AllGameData;
use super::snapshot::Snapshot;

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Never returns. `None` means no match the player is part of is running.
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

    let mut gold_earned = GoldEarned::default();
    loop {
        let data = fetch(&client, url).await;
        on_update(data.and_then(|data| snapshot_of(&data, &mut gold_earned)));
        clock.sleep(POLL_INTERVAL).await;
    }
}

async fn fetch(client: &reqwest::Client, url: &str) -> Option<AllGameData> {
    let body = client.get(url).send().await.ok()?.text().await.ok()?;
    serde_json::from_str(&body).ok()
}

fn snapshot_of(data: &AllGameData, gold_earned: &mut GoldEarned) -> Option<Snapshot> {
    if !data.match_running() {
        gold_earned.end_game();
        return None;
    }
    let player = data.active_player()?;
    Some(Snapshot {
        cs: player.scores.creep_score,
        level: player.level,
        gold_earned: gold_earned.observe(
            data.game_time(),
            data.current_gold(),
            player.item_counts(),
        ),
    })
}
