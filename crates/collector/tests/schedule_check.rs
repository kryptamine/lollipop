//! The Objective schedule against collected kills: fake timelines through to a fake `gh`.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::process::Output;

use lollipop_collector::SeedTier;

use support::build::run_build;
use support::game::{Game, Kill};
use support::release_bench::ReleaseBench;
use support::scenario::Scenario;
use tempfile::TempDir;

const AT_THE_SPAWN: [Kill; 5] = [
    (300_000, "DRAGON", Some("FIRE_DRAGON")),
    (480_000, "HORDE", None),
    (900_000, "RIFTHERALD", None),
    (1_200_000, "BARON_NASHOR", None),
    (1_560_000, "DRAGON", Some("ELDER_DRAGON")),
];
const EARLY_BARON: Kill = (1_180_000, "BARON_NASHOR", None);

async fn collected(games: &[Game]) -> Scenario {
    let scenario = Scenario::start(SeedTier::Gold).await;
    scenario.riot.add_seed("GOLD", "seed", games);
    let count = i64::try_from(games.len()).unwrap();
    scenario.run_until_all_visited(count).await;
    scenario
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn titles(bench: &ReleaseBench) -> Vec<String> {
    let issues = bench.issues();
    let titles = issues.iter().map(|issue| issue.split(": ").next().unwrap());
    titles.map(str::to_string).collect()
}

#[tokio::test]
async fn a_baron_killed_before_its_spawn_opens_an_issue() {
    let scenario = collected(&[Game::killing("EUW1_1", &[EARLY_BARON])]).await;
    let bench = ReleaseBench::new();

    let output = bench.publish_built(&scenario.database(), &[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("Baron on 16.21 at 19:40, before 20:00"));
    let issues = bench.issues();
    assert_eq!(issues.len(), 1);
    assert!(issues[0].starts_with("Baron killed before its scheduled spawn on patch 16.21"));
    assert!(issues[0].contains("at 19:40"));
    assert!(issues[0].contains("before 20:00"));
}

// A kill right on the spawn is the fastest a normal game can be, so it must stay quiet.
#[tokio::test]
async fn a_normal_timeline_opens_no_issue() {
    let scenario = collected(&[Game::killing("EUW1_1", &AT_THE_SPAWN)]).await;
    let bench = ReleaseBench::new();

    let output = bench.publish_built(&scenario.database(), &[]);

    assert!(output.status.success());
    assert!(stdout(&output).contains("first kills compared: 5, before their spawn: 0"));
    assert_eq!(bench.issues(), Vec::<String>::new());
    assert!(stdout(&output).contains("published table 1"));
}

// Elder's bound comes from the drake timers, and a moved one must not pass unseen.
#[tokio::test]
async fn every_objective_has_its_own_spawn() {
    let kills = AT_THE_SPAWN.map(|(at, monster, sub_type)| (at - 1, monster, sub_type));
    let scenario = collected(&[Game::killing("EUW1_1", &kills)]).await;
    let bench = ReleaseBench::new();

    bench.publish_built(&scenario.database(), &[]);

    let on_the_patch =
        |objective: &str| format!("{objective} killed before its scheduled spawn on patch 16.21");
    let mut expected = ["Drake", "Voidgrubs", "Herald", "Baron", "Elder"].map(on_the_patch);
    expected.sort();
    let mut opened = titles(&bench);
    opened.sort();
    assert_eq!(opened, expected);
}

// Riot's client serves an empty sub type where Match-V5 leaves the field out.
#[tokio::test]
async fn an_empty_sub_type_reads_as_none() {
    let baron = (1_180_000, "BARON_NASHOR", Some(""));
    let scenario = collected(&[Game::killing("EUW1_1", &[baron])]).await;
    let bench = ReleaseBench::new();

    bench.publish_built(&scenario.database(), &[]);

    assert_eq!(
        titles(&bench),
        ["Baron killed before its scheduled spawn on patch 16.21"]
    );
}

// One game in thousands is enough to show a timer moved.
#[tokio::test]
async fn the_earliest_kill_of_any_game_on_the_patch_counts() {
    let scenario = collected(&[
        Game::killing("EUW1_1", &AT_THE_SPAWN),
        Game::killing("EUW1_2", &[EARLY_BARON]),
        Game::killing("EUW1_3", &[(1_300_000, "BARON_NASHOR", None)]),
    ])
    .await;
    let bench = ReleaseBench::new();

    bench.publish_built(&scenario.database(), &[]);

    assert_eq!(bench.issues().len(), 1);
}

#[tokio::test]
async fn each_patch_gets_its_own_issue() {
    let older = Game {
        version: String::from("16.20.1.1"),
        ..Game::killing("EUW1_1", &[EARLY_BARON])
    };
    let scenario = collected(&[older, Game::killing("EUW1_2", &[EARLY_BARON])]).await;
    let bench = ReleaseBench::new();

    bench.publish_built(&scenario.database(), &[]);

    assert_eq!(
        titles(&bench),
        [
            "Baron killed before its scheduled spawn on patch 16.20",
            "Baron killed before its scheduled spawn on patch 16.21",
        ]
    );
}

// The weekly build sees the same kill until its patch leaves the window.
#[tokio::test]
async fn a_later_build_does_not_repeat_the_issue() {
    let scenario = collected(&[Game::killing("EUW1_1", &[EARLY_BARON])]).await;
    let bench = ReleaseBench::new();

    bench.publish_built(&scenario.database(), &[]);
    let again = bench.publish_built(&scenario.database(), &[]);

    assert!(again.status.success());
    assert_eq!(bench.issues().len(), 1);
}

// A new Season has no schedule to break, and an unknown type may be a renamed one to look at.
#[tokio::test]
async fn what_the_schedule_does_not_cover_is_reported_with_no_issue() {
    let next_season = Game {
        version: String::from("17.1.1.1"),
        ..Game::killing("EUW1_1", &[EARLY_BARON])
    };
    let scenario = collected(&[next_season]).await;
    let bench = ReleaseBench::new();
    let output = bench.publish_built(&scenario.database(), &[]);
    assert!(stdout(&output).contains("not compared: BARON_NASHOR on 17.1"));
    assert_eq!(bench.issues(), Vec::<String>::new());

    let scenario = collected(&[Game::killing("EUW1_1", &[(60_000, "ATAKHAN", None)])]).await;
    let bench = ReleaseBench::new();
    let output = bench.publish_built(&scenario.database(), &[]);
    assert!(stdout(&output).contains("not compared: ATAKHAN on 16.21"));
    assert_eq!(bench.issues(), Vec::<String>::new());
}

// Losing the notice would lose the only signal of a moved timer until someone looks.
#[tokio::test]
async fn a_notice_that_cannot_open_fails_the_publish() {
    let scenario = collected(&[Game::killing("EUW1_1", &[EARLY_BARON])]).await;
    let bench = ReleaseBench::new();
    bench.break_gh("issue-list");

    let output = bench.publish_built(&scenario.database(), &[]);

    assert!(!output.status.success());
    assert_eq!(bench.asset_names(), Vec::<String>::new());
    assert!(bench.issues()[0].starts_with("Benchmark table publish failed"));
}

// A manual build stays offline, as it has no Release to publish to.
#[tokio::test]
async fn build_table_prints_the_check_and_opens_nothing() {
    let scenario = collected(&[Game::killing("EUW1_1", &[EARLY_BARON])]).await;
    let bench = ReleaseBench::new();
    let dir = TempDir::new().unwrap();

    let build = run_build(&scenario.database(), &bench.reworks(), dir.path(), &[]);

    assert!(build
        .stdout()
        .contains("Baron on 16.21 at 19:40, before 20:00"));
    assert_eq!(bench.issues(), Vec::<String>::new());
}
