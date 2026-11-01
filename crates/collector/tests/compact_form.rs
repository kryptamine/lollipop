//! The compact form of a match in the database.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;

use lollipop_collector::SeedTier;
use rusqlite::Connection;

use support::game::Game;
use support::scenario::Scenario;

async fn one_stored_game() -> Scenario {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario
        .riot
        .add_seed("GOLD", "seed-puuid", &[Game::new("EUW1_1")]);
    scenario.run_until(|s| s.stored() == 1).await;
    scenario
}

// A table build interpolates to the minute, which needs each frame's own late timestamp
// and its values per participant. CS is both kinds of minion.
#[tokio::test]
async fn keeps_per_participant_values_for_every_frame() {
    let scenario = one_stored_game().await;
    let connection = Connection::open(scenario.database()).unwrap();
    let row: (i64, i64, i64) = connection
        .query_row(
            "SELECT cs, total_gold, level FROM minute_values
             WHERE participant = 4 AND at_ms = 120050",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(row, (84, 1300, 3));
    assert_eq!(scenario.count("SELECT count(*) FROM minute_values"), 30);
    let support: (String, String) = connection
        .query_row(
            "SELECT champion, role FROM participants WHERE participant = 5",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        support,
        (String::from("Champion5"), String::from("UTILITY"))
    );
    assert_eq!(scenario.count("SELECT count(*) FROM participants"), 10);
}

// These get compared with the Objective schedule, where Elder runs on its own timer.
#[tokio::test]
async fn keeps_the_first_kill_time_per_objective() {
    let scenario = one_stored_game().await;
    let connection = Connection::open(scenario.database()).unwrap();
    let mut statement = connection
        .prepare("SELECT objective, at_ms FROM first_kills ORDER BY objective")
        .unwrap();
    let kills: Vec<(String, i64)> = statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let expected = [
        ("BARON_NASHOR", 100_000),
        ("DRAGON", 70_000),
        ("ELDER_DRAGON", 90_000),
        ("HORDE", 50_000),
        ("RIFTHERALD", 110_000),
    ];
    let expected = expected.map(|(objective, at_ms)| (objective.to_string(), at_ms));
    assert_eq!(kills, expected);
}

fn assert_no_identity_on_disk(dir: &Path) {
    let files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert!(files.iter().any(|file| file.ends_with("collector.sqlite3")));
    for file in files {
        let bytes = std::fs::read(&file).unwrap();
        for needle in ["puuid", "name-of"] {
            let found = bytes
                .windows(needle.len())
                .any(|part| part == needle.as_bytes());
            assert!(!found, "{needle} found in {}", file.display());
        }
    }
}

// PUUIDs are never stored, from the match, the timeline or the seed, and that includes
// the queue a restart resumes from. Reading the raw files covers every table and the WAL.
#[tokio::test]
async fn no_puuid_or_player_name_reaches_the_disk() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    let games = [Game::new("EUW1_1"), Game::new("EUW1_2")];
    scenario.riot.add_seed("GOLD", "seed-puuid", &games);
    scenario.riot.hang_after("/timeline", 1);
    scenario
        .run_until(|s| s.riot.request_count("/timeline") == 2)
        .await;
    assert_eq!(scenario.count("SELECT count(*) FROM pending_matches"), 1);
    assert_no_identity_on_disk(scenario.dir.path());

    scenario.riot.release();
    scenario.run_until(|s| s.stored() == 2).await;
    assert_no_identity_on_disk(scenario.dir.path());
}
