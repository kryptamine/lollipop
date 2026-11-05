//! The CS pace Marker against a Benchmark table on disk, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::Marker;
use lollipop_core::live::Marker::{Above, Below, Neutral};
use serde_json::Value;

use support::benchmark_file::{cell, minute, table_of, ENOUGH_GAMES};
use support::marker_game::{MarkerBench, Moment, Seat, CHAMPION, MASTER_YI_MID, ROLE, TIER};

/// 6 CS a minute at every minute of a 40 minute game.
fn steady_minutes() -> Vec<Value> {
    (0..=40)
        .map(|at| minute(at, 6.0 * f64::from(at), 1000.0, 10.0, ENOUGH_GAMES))
        .collect()
}

async fn steady_bench() -> MarkerBench {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &steady_minutes())]));
    bench
}

/// CS steps from 30 to 40 at 5:00, then reads `cs` at 8:00.
const fn three_minutes_to(cs: u32) -> [Moment; 3] {
    [
        Moment::at(290.0).cs(30),
        Moment::at(300.0).cs(40),
        Moment::at(480.0).cs(cs),
    ]
}

async fn pace_after(bench: &MarkerBench, moments: &[Moment]) -> Option<Marker> {
    bench.markers_for(moments).await.last().unwrap().cs_pace
}

// The Cell gains 6 a minute, so the zone is its floor of 2. The step at 5:00 is exact and
// the read at 8:00 counts 5 above, so 59 is 8 a minute and 47 is 4, both on the edge.
#[tokio::test]
async fn compares_cs_a_minute_since_a_step_three_minutes_back() {
    let bench = steady_bench().await;
    let cases = [(59, Neutral), (60, Above), (47, Neutral), (46, Below)];
    for (cs, expected) in cases {
        let marker = pace_after(&bench, &three_minutes_to(cs)).await;
        assert_eq!(marker, Some(expected), "CS {cs}");
    }
}

// A start at the first poll would read a floored count as exact, and an older step would
// blur a player who stopped farming into their lane phase. Since 7:00 113 is 6 a minute.
#[tokio::test]
async fn the_window_starts_at_the_latest_step_three_minutes_back() {
    let bench = steady_bench().await;
    let moments = [
        Moment::at(290.0).cs(30),
        Moment::at(300.0).cs(40),
        Moment::at(420.0).cs(100),
        Moment::at(600.0).cs(113),
    ];

    assert_eq!(pace_after(&bench, &moments).await, Some(Neutral));
}

// The first poll of a game first seen late is no step, so its CS is never counted as exact.
#[tokio::test]
async fn no_pace_until_a_step_lies_three_minutes_back() {
    let bench = steady_bench().await;
    let moments = [
        Moment::at(600.0).cs(100),
        Moment::at(610.0).cs(110),
        Moment::at(789.0).cs(120),
        Moment::at(790.0).cs(120),
    ];

    let markers = bench.markers_for(&moments).await;

    assert_eq!(
        markers.iter().map(|m| m.cs_pace).collect::<Vec<_>>(),
        [None, None, None, Some(Neutral),]
    );
}

#[tokio::test]
async fn shows_the_pace_and_the_cells_gain_over_the_same_stretch() {
    let bench = steady_bench().await;

    let shown = bench.benchmarks_for(&three_minutes_to(53)).await;

    assert_eq!(
        shown.iter().map(|b| b.cs_pace).collect::<Vec<_>>(),
        [None, None, Some(6.0)]
    );
}

// The pace is the player's own, so it shows with no Cell to compare it to.
#[tokio::test]
async fn the_pace_shows_without_a_table() {
    let bench = MarkerBench::new().await;

    let paces = bench.paces_for(&three_minutes_to(53)).await;

    assert_eq!(paces, [None, None, Some(6.0)]);
}

// Supports do not farm, so like their CS their pace is compared to nothing.
#[tokio::test]
async fn a_support_has_no_pace_marker() {
    let mut bench = MarkerBench::new().await;
    bench.seat = Seat {
        position: "UTILITY",
        ..MASTER_YI_MID
    };
    bench.write_table(&table_of(&[cell(
        CHAMPION,
        "UTILITY",
        TIER,
        &steady_minutes(),
    )]));

    let markers = bench.markers_for(&three_minutes_to(60)).await;
    let shown = bench.benchmarks_for(&three_minutes_to(60)).await;

    assert_eq!(markers[2].cs_pace, None);
    assert_eq!(shown[2].cs_pace, None);
}
