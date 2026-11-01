//! Which seed players are walked, and how the calls are made.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_collector::{Clock, SeedTier};
use rusqlite::Connection;

use support::game::Game;
use support::scenario::{Scenario, KEY};

fn tier_of(scenario: &Scenario, match_id: &str) -> String {
    let connection = Connection::open(scenario.database()).unwrap();
    connection
        .query_row(
            "SELECT tier FROM matches WHERE match_id = ?",
            [match_id],
            |row| row.get(0),
        )
        .unwrap()
}

// The private phase collects only these three, for about three times the games per Cell.
#[tokio::test]
async fn walks_the_owners_tier_and_its_neighbours_only() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    for (tier, id) in [
        ("SILVER", "EUW1_S"),
        ("GOLD", "EUW1_G"),
        ("PLATINUM", "EUW1_P"),
    ] {
        scenario
            .riot
            .add_seed(tier, &format!("seed-{id}"), &[Game::new(id)]);
    }
    scenario
        .riot
        .add_seed("BRONZE", "seed-bronze", &[Game::new("EUW1_B")]);
    scenario.run_until(|s| s.stored() == 3).await;

    // The seed's tier labels the whole match.
    assert_eq!(tier_of(&scenario, "EUW1_S"), "SILVER");
    assert_eq!(tier_of(&scenario, "EUW1_G"), "GOLD");
    assert_eq!(tier_of(&scenario, "EUW1_P"), "PLATINUM");
    assert_eq!(scenario.riot.request_count("/BRONZE/"), 0);
    assert_eq!(scenario.riot.request_count("leagues/"), 0);
}

#[tokio::test]
async fn the_lowest_tier_has_one_neighbour() {
    let scenario = Scenario::start(SeedTier::Iron).await;
    scenario
        .riot
        .add_seed("IRON", "seed-iron", &[Game::new("EUW1_I")]);
    scenario
        .riot
        .add_seed("BRONZE", "seed-bronze", &[Game::new("EUW1_B")]);
    scenario
        .riot
        .add_seed("SILVER", "seed-silver", &[Game::new("EUW1_S")]);
    scenario.run_until(|s| s.stored() == 2).await;
    assert_eq!(scenario.riot.request_count("/SILVER/"), 0);
}

// Master and above is one tier of a Cell, read from three apex leagues with no divisions.
#[tokio::test]
async fn merges_the_apex_leagues_into_one_tier() {
    let scenario = Scenario::start(SeedTier::MasterPlus).await;
    for (league, id) in [
        ("MASTER", "EUW1_M"),
        ("GRANDMASTER", "EUW1_GM"),
        ("CHALLENGER", "EUW1_C"),
    ] {
        scenario
            .riot
            .add_seed(league, &format!("seed-{id}"), &[Game::new(id)]);
    }
    scenario
        .riot
        .add_seed("DIAMOND", "seed-diamond", &[Game::new("EUW1_D")]);
    scenario.run_until(|s| s.stored() == 4).await;

    for id in ["EUW1_M", "EUW1_GM", "EUW1_C"] {
        assert_eq!(tier_of(&scenario, id), "MASTER+");
    }
    assert_eq!(tier_of(&scenario, "EUW1_D"), "DIAMOND");
    assert_eq!(
        scenario
            .riot
            .request_count("/entries/RANKED_SOLO_5x5/MASTER"),
        0
    );
}

// The page count is unknown and random pages overshoot it, which must not end the walk.
#[tokio::test]
async fn finds_seeds_when_most_pages_are_empty() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario.riot.have_one_page();
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    scenario.run_until(|s| s.stored() == 1).await;
}

// Riot answers 403 to a client with no User-Agent of its own.
#[tokio::test]
async fn every_call_carries_the_key_and_a_user_agent() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    scenario.run_until(|s| s.stored() == 1).await;
    for request in scenario.riot.requests() {
        assert_eq!(request.token.as_deref(), Some(KEY));
        assert!(request
            .user_agent
            .unwrap()
            .starts_with("lollipop-collector/"));
    }
}

// Source is ranked solo only. startTime keeps the calls off patches long out of the window.
#[tokio::test]
async fn asks_for_recent_ranked_solo_games() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed", &[Game::new("EUW1_1")]);
    let four_weeks_ago_s = (scenario.clock.now_ms() - 28 * 24 * 60 * 60 * 1000) / 1000;
    scenario.run_until(|s| s.stored() == 1).await;
    let requests = scenario.riot.requests();
    let ids = requests.iter().find(|r| r.path.contains("/ids")).unwrap();
    assert!(ids.path.contains("queue=420"), "{}", ids.path);
    assert!(
        ids.path.contains(&format!("startTime={four_weeks_ago_s}")),
        "{}",
        ids.path
    );
}

// Up to 3 games share a seed, as more would make the sample less independent. The rest
// stay reachable on a later visit, since only what was picked counts as visited.
#[tokio::test]
async fn takes_at_most_three_matches_per_seed_per_visit() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    let games: Vec<Game> = (0..5).map(|n| Game::new(&format!("EUW1_{n}"))).collect();
    scenario.riot.add_seed("GOLD", "seed", &games);
    scenario.run_until(|s| s.stored() == 5).await;

    let requests = scenario.riot.requests();
    let mut visits = requests.split(|r| r.path.contains("/ids")).skip(1);
    let first_visit = visits.next().unwrap();
    let timelines = first_visit.iter().filter(|r| r.path.contains("/timeline"));
    assert_eq!(timelines.count(), 3);
}
