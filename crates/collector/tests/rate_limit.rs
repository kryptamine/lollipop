//! The application rate limit, within a run and across a restart.
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

use lollipop_collector::{Clock, SeedTier};

use support::failure::Failure;
use support::game::Game;
use support::scenario::Scenario;

async fn scenario_with_many_games(limit: &str) -> Scenario {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario.riot.set_rate_limit(limit);
    for seed in 0..20 {
        let games: Vec<Game> = (0..3)
            .map(|n| Game::new(&format!("EUW1_{seed}{n}")))
            .collect();
        scenario
            .riot
            .add_seed("GOLD", &format!("seed-{seed}"), &games);
    }
    scenario
}

fn assert_within(scenario: &Scenario, max_calls: usize, window: Duration) {
    let window = u64::try_from(window.as_millis()).unwrap();
    let times: Vec<u64> = scenario.riot.requests().iter().map(|r| r.at_ms).collect();
    for (index, start) in times.iter().enumerate() {
        let inside = times[index..]
            .iter()
            .filter(|t| **t < start + window)
            .count();
        assert!(
            inside <= max_calls,
            "{inside} calls in a window of {window} ms"
        );
    }
}

fn elapsed(scenario: &Scenario) -> Duration {
    let requests = scenario.riot.requests();
    let first = requests.first().unwrap().at_ms;
    Duration::from_millis(requests.last().unwrap().at_ms - first)
}

// Held at 90% of Riot's count, and the elapsed bound fails a limiter that over-waits.
#[tokio::test]
async fn stays_inside_every_window_it_is_told_about() {
    let scenario = scenario_with_many_games("10:1,30:60").await;
    scenario.run_until(|s| s.riot.requests().len() >= 60).await;
    assert_within(&scenario, 9, Duration::from_secs(1));
    assert_within(&scenario, 27, Duration::from_secs(60));
    assert!(
        elapsed(&scenario) >= Duration::from_secs(60),
        "the limit never bound"
    );
    assert!(
        elapsed(&scenario) < Duration::from_secs(180),
        "{:?}",
        elapsed(&scenario)
    );
}

// Restarting with a window still open is what earns an application 429.
#[tokio::test]
async fn a_restart_does_not_reopen_a_window() {
    let scenario = scenario_with_many_games("10:1,30:60").await;
    for answered in [25, 50, 60] {
        scenario.riot.hang_after("/lol/", answered);
        scenario
            .run_until(|s| s.riot.requests().len() == answered + 1)
            .await;
        scenario.riot.release();
    }
    assert_within(&scenario, 27, Duration::from_secs(60));
}

// Riot's shared service limit answers 429 too, and no local count can foresee it.
#[tokio::test]
async fn waits_out_retry_after_and_retries_the_same_call() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    let failure = Failure {
        status: 429,
        retry_after_s: Some(30),
    };
    scenario.riot.fail("/EUW1_1/timeline", failure, 1);
    scenario.run_until(|s| s.stored() == 1).await;

    let requests = scenario.riot.requests();
    let timeline: Vec<u64> = requests
        .iter()
        .filter(|r| r.path.contains("/timeline"))
        .map(|r| r.at_ms)
        .collect();
    assert_eq!(timeline.len(), 2);
    assert!(timeline[1] - timeline[0] >= 30_000);
}

#[tokio::test]
async fn a_restart_does_not_cut_a_retry_after_short() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    let failure = Failure {
        status: 429,
        retry_after_s: Some(30),
    };
    scenario.riot.fail("/EUW1_1/timeline", failure, 1);
    let limited_at = scenario.clock.now_ms();
    scenario.clock.set_frozen(true);
    scenario
        .run_until(|s| {
            s.count("SELECT count(*) FROM collector_state WHERE key = 'calls_held_until'") == 1
        })
        .await;
    scenario.clock.set_frozen(false);

    scenario.run_until(|s| s.stored() == 1).await;
    let requests = scenario.riot.requests();
    let mut timeline = requests.iter().filter(|r| r.path.contains("/timeline"));
    assert!(timeline.next_back().unwrap().at_ms >= limited_at + 30_000);
}

// A 429 with no Retry-After is what Riot's service limit can send.
#[tokio::test]
async fn a_429_with_no_retry_after_still_waits() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    let start = scenario.clock.now_ms();
    let failure = Failure {
        status: 429,
        retry_after_s: None,
    };
    scenario.riot.fail("/EUW1_1/timeline", failure, 1);
    scenario.run_until(|s| s.stored() == 1).await;
    assert!(scenario.clock.now_ms() - start >= 10_000);
}
