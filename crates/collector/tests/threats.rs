//! What enemy teams heal and mitigate, and the Counter items that answer it, built by the
//! binary from archived matches and Data Dragon.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use serde_json::json;

use support::archived_match::{damage, ArchivedMatch};
use support::fake_data_dragon::{LORD_DOMINIKS, MORELLONOMICON, MORTAL_REMINDER, VOID_STAFF};
use support::played::Played;
use support::table_bench::TableBench;

/// Soraka heals 30,000 in a 30 minute game and 20,000 in a 20 minute one, and Malphite
/// mitigates 60,000 in the first; every other seat is a Filler who does neither. A game
/// with no length, where Soraka heals a lot, counts nowhere.
fn bench_with_soraka_and_malphite() -> TableBench {
    let mut bench = TableBench::new();
    let first = ArchivedMatch::new()
        .seat(1, damage("Soraka", 8000))
        .threats(1, 30_000, 0)
        .seat(6, damage("Malphite", 6000))
        .threats(6, 0, 60_000);
    bench.add_archived(1, &first);
    let second = ArchivedMatch::new()
        .lasting(20)
        .seat(1, damage("Soraka", 8000))
        .threats(1, 20_000, 0);
    bench.add_archived(1, &second);
    let untimed = ArchivedMatch::new()
        .lasting(0)
        .seat(1, damage("Soraka", 8000))
        .threats(1, 90_000, 0);
    bench.add_archived(1, &untimed);
    bench
}

// The app sums the five enemies' amounts and compares that to the bound: both ends must
// agree on what an amount and a bound mean.
#[test]
fn each_threat_holds_a_champions_amount_a_minute_and_the_top_quarter_bound() {
    let build = bench_with_soraka_and_malphite().build(&[]);

    // Team sums 0, 0, 1000 and 1000 put the bound at 1000 for healing; 0, 0, 0 and 2000
    // put it at 500 for mitigation.
    assert_eq!(
        build.table()["threats"],
        json!({
            "healing": {
                "per_minute": { "Filler": 0.0, "Malphite": 0.0, "Soraka": 1000.0 },
                "heavy_from": 1000.0,
            },
            "mitigation": {
                "per_minute": { "Filler": 0.0, "Malphite": 2000.0, "Soraka": 0.0 },
                "heavy_from": 500.0,
            },
        })
    );
}

// Grievous Wounds has no Data Dragon tag, and a component is never in a Build.
#[test]
fn counters_are_the_completed_items_that_wound_or_penetrate() {
    let build = bench_with_soraka_and_malphite().build(&[]);

    assert_eq!(
        build.table()["counters"],
        json!({
            "anti_heal": [MORTAL_REMINDER, MORELLONOMICON],
            "armor_penetration": [MORTAL_REMINDER, LORD_DOMINIKS],
            "magic_penetration": [VOID_STAFF],
        })
    );
}

#[test]
fn a_store_with_no_archived_match_has_no_threats() {
    let mut bench = TableBench::new();
    bench.add(100, &Played::new("Ahri", "GOLD"));

    let build = bench.build(&[]);

    assert!(build.table().get("threats").is_none());
}
