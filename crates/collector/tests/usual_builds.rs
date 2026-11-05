//! The usual builds in the Benchmark table: the completed items games of a champion and
//! role buy together, built by the binary from archived matches.
#![allow(
    clippy::unwrap_used,
    clippy::future_not_send,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, a test crate has no public API, and a rusqlite Connection is not Sync"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use serde_json::{json, Value};

use support::archived_match::{ArchivedMatch, Seat};
use support::build::Build;
use support::fake_data_dragon::{MERCURYS_TREADS, MORE_COMPLETED};
use support::table_bench::TableBench;

const AHRI: Seat = Seat {
    champion: "Ahri",
    role: "MIDDLE",
    magic: 9000,
    total: 10_000,
};
const SHARED: u32 = MORE_COMPLETED;
const FIRST: u32 = MORE_COMPLETED + 1;
const INSTEAD: u32 = MORE_COMPLETED + 2;

fn buying(items: &[u32]) -> ArchivedMatch {
    items.iter().zip(10..).fold(
        ArchivedMatch::new().seat(1, AHRI),
        |archived, (&item, minute)| archived.buys(1, item, minute),
    )
}

fn build(bench: &TableBench, min_games: u32) -> Build {
    bench.build(&["--min-candidate-games", &min_games.to_string()])
}

/// Every game faced the same bucket, so each bucket holds the pooled build.
fn usual_build(build: &Build) -> Value {
    let entry = &build.table()["item_candidates"][0];
    let builds = &entry["usual_builds"];
    assert_eq!(builds["physical_heavy"], builds["magic_heavy"]);
    assert_eq!(builds["mixed"], builds["magic_heavy"]);
    builds["mixed"].clone()
}

/// Six games open on one item and four on another instead, and all ten buy a third.
fn bench_with_two_openings() -> TableBench {
    let mut bench = TableBench::new();
    bench.add_archived(6, &buying(&[FIRST, SHARED]));
    bench.add_archived(4, &buying(&[INSTEAD, SHARED]));
    bench
}

// Shares counted one item at a time put both openings in one Build, which nobody buys.
#[test]
fn an_item_bought_instead_of_another_stays_out_of_the_usual_build() {
    let build = build(&bench_with_two_openings(), 1);

    assert_eq!(usual_build(&build), json!([SHARED, FIRST]));
}

// Narrowing to a handful of games would let noise pick the late items.
#[test]
fn games_too_few_to_narrow_to_leave_the_rest_by_share() {
    let build = build(&bench_with_two_openings(), 7);

    assert_eq!(usual_build(&build), json!([SHARED, FIRST, INSTEAD]));
}

// The Build takes its boots by share, and holds five completed items beside them.
#[test]
fn a_usual_build_holds_five_completed_items_and_no_boots() {
    let mut bench = TableBench::new();
    let mut items: Vec<u32> = (0..7).map(|index| MORE_COMPLETED + index).collect();
    items.push(MERCURYS_TREADS);
    bench.add_archived(1, &buying(&items));

    let build = build(&bench, 1);

    let picked = usual_build(&build);
    assert_eq!(picked.as_array().unwrap().len(), 5);
    assert!(!picked.as_array().unwrap().contains(&json!(MERCURYS_TREADS)));
}
