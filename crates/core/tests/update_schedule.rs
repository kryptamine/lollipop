//! When the app checks for and installs an update, from docs/spec/mvp-core.md,
//! with League's phase faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::updates::{UpdateSchedule, UpdateStep};

use support::polling::between_games_over;

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
const ENDED: &str = r#"{
  "activePlayer": { "riotId": "Me#EUW", "currentGold": 500.0 },
  "allPlayers": [{ "riotId": "Me#EUW", "level": 1, "items": [], "scores": { "creepScore": 220 } }],
  "events": { "Events": [{ "EventName": "GameStart" }, { "EventName": "GameEnd" }] },
  "gameData": { "gameTime": 1995.2 }
}"#;

/// Feeds one poll loop over `bodies` to a fresh schedule, chronologically: the
/// step each poll starts, and a download finishing after poll `downloaded_after`.
async fn steps_over(
    bodies: Vec<Option<&str>>,
    downloaded_after: Option<usize>,
) -> Vec<Option<UpdateStep>> {
    let mut schedule = UpdateSchedule::new();
    let mut steps = Vec::new();
    for (index, between_games) in between_games_over(bodies).await.into_iter().enumerate() {
        steps.push(schedule.set_between_games(between_games));
        if downloaded_after == Some(index) {
            steps.push(schedule.update_downloaded());
        }
    }
    steps
}

const CHECK: Option<UpdateStep> = Some(UpdateStep::Check);
const INSTALL: Option<UpdateStep> = Some(UpdateStep::Install);

#[tokio::test]
async fn checks_at_launch_in_the_lobby() {
    assert_eq!(steps_over(vec![None], None).await, [CHECK]);
}

// A download competes with the game, so a launch mid-match waits for the client.
#[tokio::test]
async fn a_launch_during_a_match_checks_once_league_is_gone() {
    let bodies = vec![Some(RUNNING), Some(RUNNING), Some(ENDED), None];
    assert_eq!(steps_over(bodies, None).await, [None, None, None, CHECK]);
}

#[tokio::test]
async fn checks_again_after_each_game() {
    let bodies = vec![None, None, Some(RUNNING), Some(ENDED), None];
    let steps = steps_over(bodies, None).await;
    assert_eq!(steps, [CHECK, None, None, None, CHECK]);
}

#[tokio::test]
async fn installs_a_download_that_finishes_between_games() {
    assert_eq!(steps_over(vec![None], Some(0)).await, [CHECK, INSTALL]);
}

// Installing relaunches the app: a poll that fails mid-match must not look like its end.
#[tokio::test]
async fn a_failed_poll_mid_match_holds_the_install() {
    let bodies = vec![None, Some(RUNNING), None, Some(RUNNING), Some(ENDED), None];
    let steps = steps_over(bodies, Some(1)).await;
    assert_eq!(steps, [CHECK, None, None, None, None, None, INSTALL]);
}

#[tokio::test]
async fn the_next_game_loading_screen_holds_the_install() {
    let bodies = vec![None, Some(RUNNING), Some(ENDED), Some(LOADING)];
    let steps = steps_over(bodies, Some(1)).await;
    assert_eq!(steps, [CHECK, None, None, None, None]);
}
