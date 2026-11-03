//! The Benchmarks shown beside the Markers, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::benchmarks::ScoresAt;
use lollipop_core::live::ShownBenchmarks;
use serde_json::Value;

use support::benchmark_file::{cell, flat_minutes, minute, table_of, with_scores, ENOUGH_GAMES};
use support::marker_game::{MarkerBench, Moment, Seat, CHAMPION, MASTER_YI_MID, ROLE, TIER};

fn growing_minutes() -> Vec<Value> {
    (0..=20)
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
        .collect()
}

// A Benchmark that crept up on every poll was noise to read, and one a minute ahead read as
// falling behind. At 10:00 the Cell reads 60 CS, 4500 gold and level 6, shown through 10:59.
#[tokio::test]
async fn shows_the_benchmark_of_the_last_whole_minute() {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &growing_minutes())]));

    let shown = bench
        .benchmarks_for(&[
            Moment::at(1.0),
            Moment::at(599.0),
            Moment::at(600.0),
            Moment::at(630.0),
            Moment::at(659.0),
            Moment::at(660.0),
        ])
        .await;

    let at_ten = ShownBenchmarks {
        cs: Some(60.0),
        gold: Some(4500.0),
        level: Some(6.0),
        scores: None,
    };
    assert_eq!(shown[1].cs, Some(54.0));
    assert_eq!(&shown[2..5], [at_ten; 3]);
    assert_eq!(shown[5].cs, Some(66.0));
}

// K/D/A follows the same last-minute rule, and goes with its Marker when scores are missing.
#[tokio::test]
async fn shows_the_scores_of_the_last_whole_minute() {
    let bench = MarkerBench::new().await;
    let minutes: Vec<Value> = growing_minutes()
        .into_iter()
        .enumerate()
        .map(|(at, minute)| {
            let at = f64::from(u32::try_from(at).unwrap());
            with_scores(minute, [at / 4.0, at / 2.0, at], ENOUGH_GAMES)
        })
        .collect();
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));

    let shown = bench
        .benchmarks_for(&[Moment::at(630.0).scores(1, 1, 1), Moment::at(631.0)])
        .await;

    let at_ten = ScoresAt {
        kills: 2.5,
        deaths: 5.0,
        assists: 10.0,
    };
    assert_eq!(shown[0].scores, Some(at_ten));
    assert_eq!(shown[1].scores, None);
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
