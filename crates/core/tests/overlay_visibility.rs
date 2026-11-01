//! The visibility table in docs/spec/mvp-core.md, with League's phase faked over HTTP
//! and the frontmost app given as the bundle id the OS would report.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::overlay::{Visibility, GAME_BUNDLE_ID};
use tokio::net::TcpListener;

use support::polling::first_update;
use support::server::serve;

const CLIENT_BUNDLE_ID: &str = "com.riotgames.LeagueofLegends.LeagueClientUx";
const LOLLIPOP_BUNDLE_ID: &str = "dev.lollipop.overlay";
const OTHER_BUNDLE_ID: &str = "com.apple.finder";

const LOADING: &str = r#"{
  "activePlayer": { "riotId": "Me#EUW" },
  "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 0 } }],
  "events": { "Events": [] },
  "gameData": { "gameTime": 0.02 }
}"#;
const NO_ACTIVE_PLAYER: &str = r#"{
  "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 0 } }],
  "events": { "Events": [{ "EventName": "GameStart" }] },
  "gameData": { "gameTime": 0.02 }
}"#;
const RUNNING: &str = r#"{
  "activePlayer": { "riotId": "Me#EUW" },
  "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 60 } }],
  "events": { "Events": [{ "EventName": "GameStart" }] },
  "gameData": { "gameTime": 600.0 }
}"#;
const ENDED: &str = r#"{
  "activePlayer": { "riotId": "Me#EUW" },
  "allPlayers": [{ "riotId": "Me#EUW", "scores": { "creepScore": 220 } }],
  "events": { "Events": [{ "EventName": "GameStart" }, { "EventName": "GameEnd" }] },
  "gameData": { "gameTime": 1995.2 }
}"#;

async fn match_running(url: String) -> bool {
    first_update(url).await.is_some()
}

async fn match_running_for(body: &'static str) -> bool {
    match_running(serve(Box::new(|| Some(body.to_string()))).await).await
}

fn shown(frontmost: &str, running: bool) -> bool {
    let mut visibility = Visibility::new(GAME_BUNDLE_ID);
    visibility.set_frontmost(Some(frontmost));
    visibility.set_match_running(running);
    visibility.shown()
}

// League refuses connections while only the client lobby is up.
#[tokio::test]
async fn hidden_in_the_client_lobby() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    drop(listener);
    assert!(!shown(CLIENT_BUNDLE_ID, match_running(url.clone()).await));
    assert!(!shown(GAME_BUNDLE_ID, match_running(url).await));
}

// The game process is frontmost during loading, so only the phase can hide the overlay.
#[tokio::test]
async fn hidden_on_the_loading_screen() {
    assert!(!shown(GAME_BUNDLE_ID, match_running_for(LOADING).await));
}

#[tokio::test]
async fn a_200_with_no_active_player_counts_as_loading() {
    let running = match_running_for(NO_ACTIVE_PLAYER).await;
    assert!(!shown(GAME_BUNDLE_ID, running));
}

#[tokio::test]
async fn shown_while_the_game_is_frontmost_and_the_match_runs() {
    assert!(shown(GAME_BUNDLE_ID, match_running_for(RUNNING).await));
}

// At this level the window would otherwise draw over every app.
#[tokio::test]
async fn hidden_when_anything_else_is_frontmost() {
    let running = match_running_for(RUNNING).await;
    assert!(!shown(OTHER_BUNDLE_ID, running));
    assert!(!shown(CLIENT_BUNDLE_ID, running));
    assert!(!shown(LOLLIPOP_BUNDLE_ID, running));
}

#[tokio::test]
async fn hidden_after_game_end() {
    assert!(!shown(GAME_BUNDLE_ID, match_running_for(ENDED).await));
}

// Alt-tabbing out and back mid-match has to hide and reshow with no poll in between.
#[tokio::test]
async fn follows_the_frontmost_app_during_a_match() {
    let mut visibility = Visibility::new(GAME_BUNDLE_ID);
    visibility.set_match_running(match_running_for(RUNNING).await);
    assert!(!visibility.shown(), "nothing has been reported frontmost");

    visibility.set_frontmost(Some(GAME_BUNDLE_ID));
    assert!(visibility.shown());
    visibility.set_frontmost(Some(OTHER_BUNDLE_ID));
    assert!(!visibility.shown());
    visibility.set_frontmost(None);
    assert!(
        !visibility.shown(),
        "an app with no bundle id is not the game"
    );
    visibility.set_frontmost(Some(GAME_BUNDLE_ID));
    assert!(visibility.shown());
}
