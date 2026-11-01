//! The Benchmark table file, built by the binary from a database of matches.
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
use serde_json::json;
use tempfile::TempDir;

use support::game::Game;
use support::scenario::Scenario;
use support::table_bench::{run_build, Frame, Played, TableBench};

const AHRI: Played = Played::new("Ahri", "GOLD");

// Frames run late, and the lag grows. Taking the nearest frame as the minute would put
// every Benchmark ahead of a live game read on the exact minute.
#[test]
fn late_frames_are_interpolated_to_the_exact_minute() {
    let mut bench = TableBench::new();
    let frames: Vec<Frame> = (0..=20)
        .map(|index| Frame {
            at_ms: u64::from(index) * 61_000,
            cs: index * 61,
            total_gold: 500 + index * 610,
            level: 1 + index,
        })
        .collect();
    for _ in 0..100 {
        bench.add_game(&AHRI, &frames);
    }

    let build = bench.build(&[]);

    assert_eq!(
        build.minute("Ahri", "GOLD", 10).unwrap(),
        &json!({
            "minute": 10,
            "cs": 600.0,
            "total_gold": 6500.0,
            "level": 10.84,
            "n": 100,
        })
    );
    let first = build.minute("Ahri", "GOLD", 0).unwrap();
    assert_eq!(first["total_gold"], json!(500.0));
}

// A median stops one stomp game moving the Benchmark, and level is the one metric that is
// a mean. An even count of games takes the middle of the two central ones.
#[test]
fn a_minute_holds_median_cs_and_gold_mean_level_and_n() {
    let mut bench = TableBench::new();
    bench.add(
        50,
        &Played {
            cs_per_minute: 5,
            gold_per_minute: 300,
            level: 8,
            ..AHRI
        },
    );
    bench.add(
        49,
        &Played {
            cs_per_minute: 7,
            gold_per_minute: 400,
            level: 10,
            ..AHRI
        },
    );
    bench.add(
        1,
        &Played {
            cs_per_minute: 100,
            gold_per_minute: 9000,
            level: 18,
            ..AHRI
        },
    );

    let build = bench.build(&[]);

    assert_eq!(
        build.minute("Ahri", "GOLD", 10).unwrap(),
        &json!({
            "minute": 10,
            "cs": 60.0,
            "total_gold": 3500.0,
            "level": 9.08,
            "n": 100,
        })
    );
}

// A game ended at 20:30 says nothing about minute 21. Counting it would inflate n, and
// n decides whether a minute has a Marker.
#[test]
fn a_game_counts_only_towards_the_minutes_it_reached() {
    let mut bench = TableBench::new();
    bench.add(
        100,
        &Played {
            minutes: 20,
            ..AHRI
        },
    );
    bench.add(
        30,
        &Played {
            minutes: 30,
            ..AHRI
        },
    );

    let build = bench.build(&[]);

    assert_eq!(build.n_at("Ahri", "GOLD", 20), 130);
    assert_eq!(build.n_at("Ahri", "GOLD", 21), 30);
    assert_eq!(build.n_at("Ahri", "GOLD", 30), 30);
    assert_eq!(build.n_at("Ahri", "GOLD", 31), 0);
}

// A Cell with enough games at minute 20 stays itself, even where it thins out later:
// pooling just the late minutes is the jump the rule exists to prevent.
#[test]
fn a_cell_with_enough_games_at_minute_20_keeps_its_own_games_at_every_minute() {
    let mut bench = TableBench::new();
    bench.add(
        90,
        &Played {
            minutes: 20,
            ..AHRI
        },
    );
    bench.add(
        10,
        &Played {
            minutes: 35,
            ..AHRI
        },
    );
    bench.add(500, &Played::new("Ahri", "SILVER"));

    let build = bench.build(&[]);

    assert_eq!(build.n_at("Ahri", "GOLD", 5), 100);
    assert_eq!(build.n_at("Ahri", "GOLD", 35), 10);
}

// A Cell busy at minute 5 and thin at minute 20 takes its neighbours at every minute,
// so that minute 5 and minute 20 are read from the same games.
#[test]
fn a_thin_cell_pools_its_adjacent_tiers_at_every_minute() {
    let mut bench = TableBench::new();
    bench.add(60, &AHRI);
    bench.add(
        200,
        &Played {
            minutes: 15,
            ..AHRI
        },
    );
    bench.add(25, &Played::new("Ahri", "SILVER"));
    bench.add(25, &Played::new("Ahri", "PLATINUM"));

    let build = bench.build(&[]);

    assert_eq!(build.n_at("Ahri", "GOLD", 20), 110);
    assert_eq!(build.n_at("Ahri", "GOLD", 5), 310);
}

// A Cell with no game of its own still has adjacent tiers to read: the spec's order is
// the player's Cell, else adjacent tiers, else no Marker.
#[test]
fn a_cell_with_no_games_reads_its_adjacent_tiers() {
    let mut bench = TableBench::new();
    bench.add(60, &Played::new("Ahri", "SILVER"));
    bench.add(60, &Played::new("Ahri", "PLATINUM"));

    let build = bench.build(&[]);

    assert_eq!(build.n_at("Ahri", "GOLD", 20), 120);
}

// ADR 0003: an all-tier step would have a Master player on a rare pick always look ahead.
// Each tier here reaches one step and no further, whatever the far tiers hold.
#[test]
fn a_thin_cell_never_reads_beyond_its_adjacent_tiers() {
    let mut bench = TableBench::new();
    bench.add(60, &AHRI);
    bench.add(500, &Played::new("Ahri", "IRON"));
    bench.add(500, &Played::new("Ahri", "MASTER+"));

    let build = bench.build(&[]);

    assert!(build.cell("Ahri", "GOLD").is_none());
    assert!(build.cell("Ahri", "SILVER").is_none());
    assert!(build.cell("Ahri", "BRONZE").is_some());
    assert!(build.cell("Ahri", "EMERALD").is_none());
    assert_eq!(build.n_at("Ahri", "DIAMOND", 20), 500);
}

// One game short of the minimum is no Marker and no notice, so it has no entry at all.
#[test]
fn a_source_needs_100_games_at_minute_20() {
    let mut bench = TableBench::new();
    bench.add(99, &AHRI);
    assert!(bench.build(&[]).cell("Ahri", "GOLD").is_none());

    bench.add(1, &AHRI);
    assert!(bench.build(&[]).cell("Ahri", "GOLD").is_some());
}

// A game with no role has no Cell a player could be compared to.
#[test]
fn a_game_with_no_role_is_left_out() {
    let mut bench = TableBench::new();
    bench.add(100, &Played { role: "", ..AHRI });

    let build = bench.build(&[]);

    assert_eq!(build.table()["cells"], json!([]));
}

// The same champion in two roles is two Cells, or a mid Ahri reads a top Ahri's farm.
#[test]
fn each_role_is_a_cell_of_its_own() {
    let mut bench = TableBench::new();
    bench.add(
        100,
        &Played {
            role: "TOP",
            ..AHRI
        },
    );
    bench.add(
        100,
        &Played {
            role: "MIDDLE",
            ..AHRI
        },
    );

    let build = bench.build(&[]);

    let cells = build.table()["cells"].as_array().unwrap();
    let gold = cells.iter().filter(|cell| cell["tier"] == "GOLD");
    let roles: Vec<&str> = gold.map(|cell| cell["role"].as_str().unwrap()).collect();
    assert_eq!(roles, ["MIDDLE", "TOP"]);
    assert_eq!(build.n_at("Ahri", "GOLD", 20), 100);
}

const AATROX_OLD: Played = Played {
    patch: (16, 20),
    ..Played::new("Aatrox", "GOLD")
};
const AATROX_NEW: Played = Played {
    cs_per_minute: 9,
    ..Played::new("Aatrox", "GOLD")
};
const AATROX_REWORK: &str = r#"[{ "champion": "Aatrox", "first_valid_patch": "16.21" }]"#;

// A reworked champion's old numbers describe a different champion, so a wrong Marker is
// the cost of keeping them. Another champion on the same old patch keeps its data.
#[test]
fn a_rework_drops_that_champions_older_data_only() {
    let mut bench = TableBench::new();
    bench.add(100, &AATROX_OLD);
    bench.add(
        100,
        &Played {
            patch: (16, 20),
            ..AHRI
        },
    );
    bench.add(100, &AATROX_NEW);
    bench.set_reworks(AATROX_REWORK);

    let build = bench.build(&[]);

    let minute = build.minute("Aatrox", "GOLD", 10).unwrap();
    assert_eq!(minute["n"], json!(100));
    assert_eq!(minute["cs"], json!(90.0));
    assert_eq!(build.n_at("Ahri", "GOLD", 10), 100);
}

// The entry goes in before the patch ships, so the new patch starts with few games. The
// Marker stays away until those reach n>=100, and old data cannot fill in through a
// neighbouring tier either.
#[test]
fn a_reworked_champion_returns_at_100_games_on_the_new_patch() {
    let mut bench = TableBench::new();
    bench.add(500, &AATROX_OLD);
    bench.add(
        500,
        &Played {
            tier: "SILVER",
            ..AATROX_OLD
        },
    );
    bench.add(99, &AATROX_NEW);
    bench.set_reworks(AATROX_REWORK);
    assert!(bench.build(&[]).cell("Aatrox", "GOLD").is_none());

    bench.add(1, &AATROX_NEW);
    assert!(bench.build(&[]).cell("Aatrox", "GOLD").is_some());
}

// Going on without the list would publish a reworked champion's old Benchmark.
#[test]
fn reworks_that_do_not_read_fail_the_build_and_keep_the_old_table() {
    let mut bench = TableBench::new();
    bench.add(100, &AHRI);
    assert!(bench.build(&[]).output.status.success());
    let before = std::fs::read(bench.output()).unwrap();

    for body in [
        "not json",
        r#"[{ "champion": "Aatrox", "first_valid_patch": "soon" }]"#,
        r#"[{ "champion": "Aatrox", "first_valid_patch": "16.21" },
            { "champion": "Aatrox", "first_valid_patch": "16.22" }]"#,
    ] {
        bench.set_reworks(body);
        let build = bench.build(&[]);
        assert!(!build.output.status.success(), "{body}");
        assert!(
            build.stderr().contains("reworks.json"),
            "{}",
            build.stderr()
        );
    }
    assert_eq!(std::fs::read(bench.output()).unwrap(), before);
}

#[test]
fn a_rework_file_that_is_missing_fails_the_build() {
    let mut bench = TableBench::new();
    bench.add(100, &AHRI);
    std::fs::remove_file(bench.database().with_file_name("reworks.json")).unwrap();

    assert!(!bench.build(&[]).output.status.success());
}

const SHIFTED: Played = Played {
    cs_per_minute: 9,
    gold_per_minute: 500,
    ..AHRI
};
const OLD_AHRI: Played = Played {
    patch: (16, 20),
    ..AHRI
};

// The report exists to catch a rework nobody listed. It reports and never decides: an
// ordinary buff moves a median too, so the champion stays in the table with all its games.
#[test]
fn a_sharp_shift_is_printed_and_excludes_nobody() {
    let mut bench = TableBench::new();
    bench.add(100, &OLD_AHRI);
    bench.add(100, &SHIFTED);

    let build = bench.build(&[]);

    let stdout = build.stdout();
    assert!(stdout.contains("16.20 to 16.21"), "{stdout}");
    assert!(stdout.contains("at minute 20: 1"), "{stdout}");
    assert!(
        stdout.contains("Ahri TOP GOLD: cs 140.0 to 180.0, gold 8000 to 10000"),
        "{stdout}"
    );
    assert_eq!(build.n_at("Ahri", "GOLD", 20), 200);
}

// The bar is what a player would see: a move inside the Neutral zone changes no Marker,
// and an ordinary patch moves every median a little.
#[test]
fn a_shift_within_the_neutral_zone_is_not_sharp() {
    let mut bench = TableBench::new();
    bench.add(100, &OLD_AHRI);
    bench.add(
        100,
        &Played {
            cs_per_minute: 7,
            gold_per_minute: 430,
            ..AHRI
        },
    );

    let stdout = bench.build(&[]).stdout();

    assert!(stdout.contains("at minute 20: 0"), "{stdout}");
    assert!(!stdout.contains("Ahri"), "{stdout}");
}

#[test]
fn a_shift_on_gold_alone_past_the_neutral_zone_is_sharp() {
    let mut bench = TableBench::new();
    bench.add(100, &OLD_AHRI);
    bench.add(
        100,
        &Played {
            gold_per_minute: 440,
            ..AHRI
        },
    );

    assert!(bench.build(&[]).stdout().contains("Ahri TOP GOLD"));
}

// On a thin patch a median swings on noise, and a false alarm teaches the owner to skip it.
#[test]
fn a_shift_on_too_few_games_is_not_reported() {
    let mut bench = TableBench::new();
    bench.add(99, &OLD_AHRI);
    bench.add(300, &SHIFTED);

    assert!(bench.build(&[]).stdout().contains("at minute 20: 0"));
}

// Every build prints a report: a missing one must not read as "nothing shifted".
#[test]
fn a_build_on_one_patch_still_prints_a_report() {
    let mut bench = TableBench::new();
    bench.add(100, &AHRI);

    let stdout = bench.build(&[]).stdout();

    assert!(stdout.starts_with("drift report"), "{stdout}");
    assert!(stdout.contains("16.21"), "{stdout}");
}

// The app reads the schema version to know it can read the file, the window to know which
// patches it serves. An absent max_patch reads as no limit, so it must not be a null.
#[test]
fn the_file_carries_its_schema_version_and_patch_window() {
    let mut bench = TableBench::new();
    bench.add(100, &OLD_AHRI);
    bench.add(100, &AHRI);

    let build = bench.build(&[]);

    let table = build.table();
    assert_eq!(table["schema_version"], json!(1));
    assert_eq!(table["patch_window"], json!(["16.20", "16.21"]));
    assert!(table.get("max_patch").is_none());
}

#[test]
fn max_patch_is_carried_when_given() {
    let mut bench = TableBench::new();
    bench.add(100, &AHRI);

    let build = bench.build(&["--max-patch", "16.24"]);

    assert_eq!(build.table()["max_patch"], json!("16.24"));
}

#[test]
fn a_max_patch_that_is_not_a_patch_is_refused() {
    let mut bench = TableBench::new();
    bench.add(100, &AHRI);

    let build = bench.build(&["--max-patch", "soon"]);

    assert!(!build.output.status.success());
    assert!(!bench.output().exists());
}

// A table with no Cells would publish as "no Markers for anyone".
#[test]
fn an_empty_database_fails_the_build() {
    let bench = TableBench::new();

    let build = bench.build(&[]);

    assert!(!build.output.status.success());
    assert!(build.stderr().contains("no matches"), "{}", build.stderr());
    assert!(!bench.output().exists());
}

// A mistyped path must not create an empty database and report that it has no matches.
#[test]
fn a_missing_database_fails_without_creating_one() {
    let dir = TempDir::new().unwrap();
    let database = dir.path().join("nowhere.sqlite3");
    let reworks = dir.path().join("reworks.json");
    std::fs::write(&reworks, "[]").unwrap();

    let build = run_build(&database, &reworks, dir.path(), &[]);

    assert!(!build.output.status.success());
    assert!(!database.exists());
}

// A tier the ladder does not know would be a Cell no player can reach, and silently
// losing it would look like a thin Cell.
#[test]
fn a_stored_tier_outside_the_ladder_fails_the_build() {
    let mut bench = TableBench::new();
    bench.add(
        100,
        &Played {
            tier: "WOOD",
            ..AHRI
        },
    );

    let build = bench.build(&[]);

    assert!(!build.output.status.success());
    assert!(build.stderr().contains("WOOD"), "{}", build.stderr());
}

// Protects the seam between the two commands: what the collector really stores, in WAL
// mode and from Riot's real shapes, reads back as a table, using the repository's list.
#[tokio::test]
async fn a_table_builds_from_what_the_collector_stored() {
    let scenario = Scenario::start(SeedTier::Gold).await;
    let games: Vec<Game> = (0..100)
        .map(|n| Game::lasting(&format!("EUW1_{n}"), 20))
        .collect();
    for (index, games) in games.chunks(3).enumerate() {
        scenario
            .riot
            .add_seed("GOLD", &format!("seed-{index}"), games);
    }
    scenario.run_until(|s| s.stored() == 100).await;

    let reworks = Path::new(env!("CARGO_MANIFEST_DIR")).join("reworks.json");
    let build = run_build(&scenario.database(), &reworks, scenario.dir.path(), &[]);

    assert_eq!(
        build.minute("Champion4", "GOLD", 20).unwrap(),
        &json!({
            "minute": 20,
            "cs": 803.33,
            "total_gold": 8493.34,
            "level": 20.98,
            "n": 100,
        })
    );
    let support = build.cell("Champion5", "GOLD").unwrap();
    assert_eq!(support["role"], json!("UTILITY"));
}
