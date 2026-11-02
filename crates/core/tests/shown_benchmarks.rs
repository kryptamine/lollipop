//! The Benchmarks shown beside the Markers, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::ShownBenchmarks;
use serde_json::Value;

use support::benchmark_file::{cell, flat_minutes, minute, table_of, ENOUGH_GAMES};
use support::marker_game::{MarkerBench, Moment, Seat, CHAMPION, MASTER_YI_MID, ROLE, TIER};

// At 10:30 the Cell reads 63 CS, 4700 gold and level 6.25, halfway between its minutes.
#[tokio::test]
async fn shows_the_benchmark_at_the_current_game_time() {
    let bench = MarkerBench::new().await;
    let minutes: Vec<Value> = (0..=20)
        .map(|at| {
            let time = f64::from(at);
            minute(
                at,
                6.0 * time,
                400.0f64.mul_add(time, 500.0),
                0.5f64.mul_add(time, 1.0),
                ENOUGH_GAMES,
            )
        })
        .collect();
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));

    let shown = bench
        .benchmarks_for(&[Moment::at(1.0), Moment::at(630.0)])
        .await;

    assert_eq!(
        shown[1],
        ShownBenchmarks {
            cs: Some(63.0),
            gold: Some(4700.0),
            level: Some(6.25),
        }
    );
}

// A Benchmark with no Marker beside it would read as a comparison the strip does not make.
#[tokio::test]
async fn a_support_is_shown_no_cs_benchmark() {
    let mut bench = MarkerBench::new().await;
    bench.seat = Seat {
        position: "UTILITY",
        ..MASTER_YI_MID
    };
    bench.write_table(&table_of(&[cell(
        CHAMPION,
        "UTILITY",
        TIER,
        &flat_minutes(100.0, 1000.0, 10.0),
    )]));

    let shown = bench.benchmarks_for(&[Moment::at(1.0)]).await;

    assert_eq!(shown[0].cs, None);
    assert_eq!(shown[0].gold, Some(1000.0));
    assert_eq!(shown[0].level, Some(10.0));
}

// A game joined after 1:00 has no Gold earned count, so it has no gold Marker either.
#[tokio::test]
async fn a_game_with_no_gold_count_is_shown_no_gold_benchmark() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;

    let shown = bench.benchmarks_for(&[Moment::at(100.0)]).await;

    assert_eq!(shown[0].gold, None);
    assert_eq!(shown[0].cs, Some(100.0));
}

#[tokio::test]
async fn no_cell_means_no_benchmarks() {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_of(&[cell(
        "Warwick",
        ROLE,
        TIER,
        &flat_minutes(100.0, 1000.0, 10.0),
    )]));

    let shown = bench.benchmarks_for(&[Moment::at(1.0)]).await;

    assert_eq!(shown[0], ShownBenchmarks::default());
}
