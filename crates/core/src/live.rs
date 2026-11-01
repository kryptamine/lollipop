//! Polls Riot's Live Client Data API for the active player's own stats.

use std::future::Future;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub const LIVE_CLIENT_URL: &str = "https://127.0.0.1:2999/liveclientdata/allgamedata";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AllGameData {
    active_player: ActivePlayer,
    all_players: Vec<Player>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer {
    riot_id: Option<String>,
    summoner_name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Player {
    riot_id: Option<String>,
    summoner_name: Option<String>,
    scores: Scores,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scores {
    creep_score: u32,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Snapshot {
    pub cs: u32,
}

/// Paces the poll loop, so a replay can run on `gameTime` rather than real time.
pub trait Clock: Sync {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn sleep(&self, duration: Duration) -> impl Future<Output = ()> + Send {
        tokio::time::sleep(duration)
    }
}

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
    let cs = find_active_player(&data)?.scores.creep_score;
    Some(Snapshot { cs })
}

fn find_active_player(data: &AllGameData) -> Option<&Player> {
    let active = &data.active_player;
    data.all_players.iter().find(|p| {
        let same_riot_id = active.riot_id.is_some() && p.riot_id == active.riot_id;
        let same_name = active.summoner_name.is_some() && p.summoner_name == active.summoner_name;
        same_riot_id || same_name
    })
}
