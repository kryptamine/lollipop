//! The failures that end a run, and the ones it retries.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_collector::{CollectError, SeedTier};

use support::failure::Failure;
use support::game::Game;
use support::scenario::Scenario;

const OUTAGE: Failure = Failure {
    status: 503,
    retry_after_s: None,
};

// An expired key does not recover by waiting, so the owner must see it at once.
#[tokio::test]
async fn a_rejected_key_ends_the_run() {
    for status in [401, 403] {
        let scenario = Scenario::start(SeedTier::Gold).await;
        scenario
            .riot
            .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
        let failure = Failure {
            status,
            retry_after_s: None,
        };
        scenario.riot.fail("/lol/", failure, u32::MAX);
        let error = scenario.run_until_stopped().await;
        assert!(matches!(error, CollectError::KeyRejected(code) if code == status));
        assert_eq!(scenario.riot.requests().len(), 1);
    }
}

// The message reaches the journal, and a PUUID may only live in memory.
#[tokio::test]
async fn an_outage_ends_the_run_with_no_puuid_in_the_message() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("SILVER", "secret-puuid", &[Game::new("EUW1_1")]);
    scenario.riot.fail("/ids", OUTAGE, u32::MAX);
    let error = scenario.run_until_stopped().await;
    assert!(matches!(error, CollectError::CallFailed("match-v5:ids")));
    assert!(!error.to_string().contains("secret-puuid"));
    assert_eq!(scenario.riot.request_count("/ids"), 6);
}

#[tokio::test]
async fn a_short_outage_is_retried_and_loses_nothing() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    scenario.riot.fail("/EUW1_1/timeline", OUTAGE, 3);
    scenario.run_until(|s| s.stored() == 1).await;
    assert_eq!(scenario.riot.request_count("/EUW1_1/timeline"), 4);
}

// A refused connection is an outage too, and the owner sees it rather than a silent loop.
#[tokio::test]
async fn an_unreachable_host_ends_the_run() {
    let mut scenario = Scenario::start(SeedTier::Gold).await;
    scenario.point_at("http://127.0.0.1:9");
    let error = scenario.run_until_stopped().await;
    assert!(matches!(error, CollectError::CallFailed(_)));
}
