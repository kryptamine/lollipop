//! Which games are kept, and which calls a dropped one costs.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::time::Duration;

use lollipop_collector::SeedTier;

use support::failure::Failure;
use support::game::Game;
use support::scenario::Scenario;

async fn scenario_with(games: &[Game]) -> Scenario {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario.riot.add_seed("GOLD", "seed", games);
    scenario
}

// `endOfGameResult` reads GameComplete for these, so only this flag removes them, and
// a single player carries it. The timeline call is the cost a dropped game must not pay.
#[tokio::test]
async fn drops_an_early_surrender_without_a_timeline_call() {
    let game = Game {
        early_surrender: true,
        ..Game::new("EUW1_1")
    };
    let scenario = scenario_with(&[game]).await;
    scenario.run_until_all_visited(1).await;
    assert_eq!((scenario.stored(), scenario.dropped()), (0, 1));
    assert_eq!(scenario.riot.request_count("/timeline"), 0);
}

// Games under 15 minutes exist that carry no surrender flag.
#[tokio::test]
async fn drops_a_game_under_15_minutes_and_keeps_one_of_exactly_15() {
    let short = Game {
        duration_s: 899,
        ..Game::new("EUW1_SHORT")
    };
    let long_enough = Game {
        duration_s: 900,
        ..Game::new("EUW1_LONG")
    };
    let scenario = scenario_with(&[short, long_enough]).await;
    scenario.run_until_all_visited(2).await;
    assert_eq!(scenario.stored_ids(), [String::from("EUW1_LONG")]);
    assert_eq!(scenario.riot.request_count("EUW1_SHORT/timeline"), 0);
}

#[tokio::test]
async fn drops_a_game_of_another_queue() {
    let flex = Game {
        queue: 440,
        ..Game::new("EUW1_1")
    };
    let scenario = scenario_with(&[flex]).await;
    scenario.run_until_all_visited(1).await;
    assert_eq!(scenario.stored(), 0);
}

// A match can have no timeline, and then it has no minute values to store.
#[tokio::test]
async fn drops_a_game_with_no_timeline() {
    let game = Game {
        has_timeline: false,
        ..Game::new("EUW1_1")
    };
    let scenario = scenario_with(&[game]).await;
    scenario.run_until_all_visited(1).await;
    assert_eq!((scenario.stored(), scenario.dropped()), (0, 1));
}

#[tokio::test]
async fn drops_a_game_whose_version_does_not_read() {
    let game = Game::on_patch("EUW1_1", "not-a-version");
    let scenario = scenario_with(&[game]).await;
    scenario.run_until_all_visited(1).await;
    assert_eq!((scenario.stored(), scenario.dropped()), (0, 1));
}

// One bad match must not stall the walk, or every restart would die on it again.
#[tokio::test]
async fn gives_up_on_a_match_that_keeps_failing_and_goes_on() {
    let games = [Game::new("EUW1_BAD"), Game::new("EUW1_GOOD")];
    let scenario = scenario_with(&games).await;
    let failure = Failure {
        status: 500,
        retry_after_s: None,
    };
    scenario.riot.fail("/EUW1_BAD", failure, u32::MAX);
    scenario.run_until_all_visited(2).await;
    assert_eq!(scenario.stored_ids(), [String::from("EUW1_GOOD")]);
    assert_eq!(scenario.riot.request_count("/EUW1_BAD"), 6);
}

#[tokio::test]
async fn drops_a_game_whose_body_does_not_read() {
    let scenario = scenario_with(&[Game::new("EUW1_1")]).await;
    scenario.riot.answer_unreadable("/EUW1_1");
    scenario.run_until_all_visited(1).await;
    assert_eq!((scenario.stored(), scenario.dropped()), (0, 1));
    assert_eq!(scenario.riot.request_count("/EUW1_1"), 1);
}

// A dropped id is not fetched twice while ids can still return it, and once they cannot
// its row goes, so the table does not grow for ever.
#[tokio::test]
async fn remembers_a_dropped_match_only_for_the_lookback() {
    let game = Game {
        duration_s: 60,
        ..Game::new("EUW1_1")
    };
    let scenario = scenario_with(&[game]).await;
    scenario.run_until_all_visited(1).await;
    let ids_seen = scenario.riot.request_count("/ids");
    scenario
        .run_until(|s| s.riot.request_count("/ids") > ids_seen + 2)
        .await;
    assert_eq!(scenario.riot.request_count("/EUW1_1"), 1);

    scenario
        .clock
        .advance(Duration::from_secs(29 * 24 * 60 * 60));
    scenario
        .run_until(|s| s.riot.request_count("/EUW1_1") == 2)
        .await;
}
