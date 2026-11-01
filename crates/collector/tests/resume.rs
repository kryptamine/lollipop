//! An interrupted run picks up where it stopped, with nothing lost or fetched again.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_collector::SeedTier;

use support::game::Game;
use support::scenario::Scenario;

async fn scenario_with(ids: &[&str]) -> Scenario {
    let scenario = Scenario::start(SeedTier::Gold).await;
    let games: Vec<Game> = ids.iter().map(|id| Game::new(id)).collect();
    scenario.riot.add_seed("GOLD", "seed", &games);
    scenario
}

// Protects the calls already spent: a restart that fetched stored games again would pay
// for them twice each time the VPS reboots.
#[tokio::test]
async fn a_stored_match_is_not_fetched_again_after_a_restart() {
    let scenario = scenario_with(&["EUW1_1", "EUW1_2", "EUW1_3"]).await;
    scenario.riot.hang_after("/timeline", 1);
    scenario
        .run_until(|s| s.riot.request_count("/timeline") == 2)
        .await;
    let stored = scenario.stored_ids();
    assert_eq!(stored.len(), 1);
    let before = scenario.riot.request_count(&format!("/{}", stored[0]));

    scenario.riot.release();
    scenario.run_until(|s| s.stored() == 3).await;
    let after = scenario.riot.request_count(&format!("/{}", stored[0]));
    assert_eq!(after, before);
}

// A seed's PUUID may not be stored, so what its call paid for survives as match ids, and
// the restart spends nothing on a new seed until those are done.
#[tokio::test]
async fn a_queued_match_is_fetched_first_after_a_restart() {
    let scenario = scenario_with(&["EUW1_1", "EUW1_2", "EUW1_3"]).await;
    scenario.riot.hang_after("/timeline", 1);
    scenario
        .run_until(|s| s.riot.request_count("/timeline") == 2)
        .await;
    assert_eq!(scenario.stored(), 1);
    assert_eq!(scenario.count("SELECT count(*) FROM pending_matches"), 2);
    let earlier_requests = scenario.riot.requests().len();

    scenario.riot.release();
    scenario.run_until(|s| s.stored() == 3).await;
    let resumed = &scenario.riot.requests()[earlier_requests..];
    let first_seed_call = resumed.iter().position(|r| r.path.contains("/entries/"));
    let matches = resumed[..first_seed_call.unwrap_or(resumed.len())].iter();
    assert_eq!(matches.filter(|r| r.path.contains("/timeline")).count(), 2);
    assert_eq!(scenario.count("SELECT count(*) FROM pending_matches"), 0);
}

// A kill between the two calls of a match leaves no half a match behind.
#[tokio::test]
async fn a_kill_between_match_and_timeline_loses_nothing() {
    let scenario = scenario_with(&["EUW1_1"]).await;
    scenario.riot.hang_after("/timeline", 0);
    scenario
        .run_until(|s| s.riot.request_count("/timeline") == 1)
        .await;
    assert_eq!(scenario.stored(), 0);
    assert_eq!(scenario.count("SELECT count(*) FROM participants"), 0);
    assert_eq!(scenario.count("SELECT count(*) FROM pending_matches"), 1);

    scenario.riot.release();
    scenario.run_until(|s| s.stored() == 1).await;
    assert_eq!(scenario.count("SELECT count(*) FROM pending_matches"), 0);
    assert_eq!(scenario.count("SELECT count(*) FROM participants"), 10);
}

// With restarts back to the first tier, a crash loop would starve its neighbours' Cells.
#[tokio::test]
async fn the_turn_of_the_tiers_outlives_a_restart() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    for turn in 0..4 {
        scenario.riot.hang_after("/entries/", turn);
        scenario
            .run_until(|s| s.riot.request_count("/entries/") == turn + 1)
            .await;
        scenario.riot.release();
    }
    let requests = scenario.riot.requests();
    let walked: Vec<&str> = requests.iter().map(|r| r.path.as_str()).collect();
    for (path, tier) in walked.iter().zip(["SILVER", "GOLD", "PLATINUM", "SILVER"]) {
        assert!(path.contains(&format!("/{tier}/")), "{walked:?}");
    }
}
