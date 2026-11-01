//! The menu's status line, with League's phase faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::status_line;

use support::polling::first_update;
use support::server::serve;

const RUNNING: &str = r#"{
  "activePlayer": { "riotId": "Me#EUW", "currentGold": 500.0 },
  "allPlayers": [{ "riotId": "Me#EUW", "level": 1, "items": [], "scores": { "creepScore": 60 } }],
  "events": { "Events": [{ "EventName": "GameStart" }] },
  "gameData": { "gameTime": 600.0 }
}"#;
const LOADING: &str = r#"{
  "activePlayer": { "riotId": "Me#EUW", "currentGold": 500.0 },
  "allPlayers": [{ "riotId": "Me#EUW", "level": 1, "items": [], "scores": { "creepScore": 0 } }],
  "events": { "Events": [] },
  "gameData": { "gameTime": 0.02 }
}"#;

async fn status_for(body: &'static str) -> &'static str {
    let url = serve(Box::new(|| Some(body.to_string()))).await;
    status_line(first_update(url).await.is_some())
}

// With no table shipped yet, this is the only reason an in-game player sees no Marker.
#[tokio::test]
async fn a_running_match_gives_the_reason_for_the_missing_markers() {
    assert_eq!(status_for(RUNNING).await, "In game · No Benchmark table");
}

#[tokio::test]
async fn a_loading_game_is_not_yet_a_game() {
    assert_eq!(status_for(LOADING).await, "No game running");
}
