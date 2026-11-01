//! The Stale table rule and the strip's notices, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::LiveGame;
use serde_json::{json, Value};
use tempfile::TempDir;

use support::benchmark_file::{cell, flat_minutes, table_of};
use support::fake_lcu::{ranked, FakeLcu, RANKED, SESSION, VERSION};
use support::game_clock::GameClock;
use support::marker_game::{body, MarkerBench, Moment, Seat, CHAMPION, MASTER_YI_MID, ROLE, TIER};
use support::polling::{serve_in_order, start_playing};

const STALE_16_23: &str = "Benchmarks out of date (patch 16.23)";

fn seat_cell() -> Value {
    cell(CHAMPION, ROLE, TIER, &flat_minutes(100.0, 500.0, 10.0))
}

fn table_serving(window: &[&str], max_patch: Option<&str>) -> String {
    let mut table = json!({
        "schema_version": 1,
        "patch_window": window,
        "cells": [seat_cell()],
    });
    if let Some(max_patch) = max_patch {
        table["max_patch"] = json!(max_patch);
    }
    table.to_string()
}

fn on_patch(lcu: &FakeLcu, patch: &str) {
    lcu.set(VERSION, &format!(r#""{patch}.1+branch.code.public""#));
}

fn in_queue(lcu: &FakeLcu, queue: u32) {
    lcu.set(
        SESSION,
        &json!({ "gameData": { "queue": { "id": queue } } }).to_string(),
    );
}

async fn games_at(bench: &MarkerBench, times: &[f64]) -> Vec<LiveGame> {
    let bodies = times
        .iter()
        .map(|at| Some(body(bench.seat, Moment::at(*at), &["GameStart"])))
        .collect();
    let url = serve_in_order(bodies).await;
    let clock = GameClock::starting_at(0.0);
    let mut updates = start_playing(url, &bench.lcu.lockfile(), &bench.table(), clock);
    let mut games = Vec::with_capacity(times.len());
    for _ in times {
        games.push(updates.recv().await.unwrap().1.unwrap());
    }
    games
}

async fn notices_at_start(bench: &MarkerBench) -> Vec<String> {
    games_at(bench, &[1.0]).await[0].shown_notices()
}

async fn a_bench_with_a_table() -> MarkerBench {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_of(&[seat_cell()]));
    bench
}

// Both halves of the rule travel together: a line with Markers still up would lie.
async fn assert_serves(window: &[&str], max_patch: Option<&str>, patch: &str, serves: bool) {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_serving(window, max_patch));
    on_patch(&bench.lcu, patch);

    let moments = [Moment::at(100.0).cs(95), Moment::at(130.0).cs(95)];
    let cs = bench.markers_for(&moments).await[1].cs;
    assert_eq!(cs.is_some(), serves, "the Marker on patch {patch}");

    let stale = format!("Benchmarks out of date (patch {patch})");
    let notices = notices_at_start(&bench).await;
    assert_eq!(!notices.contains(&stale), serves, "the line on {patch}");
}

#[tokio::test]
async fn a_table_serves_its_patch_window_and_the_next_patch() {
    let window = ["16.20", "16.21"];
    assert_serves(&window, None, "16.20", true).await;
    assert_serves(&window, None, "16.21", true).await;
    assert_serves(&window, None, "16.22", true).await;
    assert_serves(&window, None, "16.23", false).await;
    // Older than the window: its Benchmarks describe a later game than the one played.
    assert_serves(&window, None, "16.19", false).await;
}

#[tokio::test]
async fn the_next_patch_crosses_a_season() {
    let window = ["16.23", "16.24"];
    assert_serves(&window, None, "17.1", true).await;
    assert_serves(&window, None, "17.2", false).await;
}

// A season patch that changes the game is why a table may be cut short ahead of it.
#[tokio::test]
async fn max_patch_overrides_the_next_patch() {
    let window = ["16.20", "16.21"];
    assert_serves(&window, Some("16.21"), "16.21", true).await;
    assert_serves(&window, Some("16.21"), "16.22", false).await;
    assert_serves(&window, Some("16.24"), "16.22", true).await;
}

#[tokio::test]
async fn a_clean_game_has_no_notice() {
    let bench = a_bench_with_a_table().await;
    assert_eq!(notices_at_start(&bench).await, Vec::<String>::new());
}

#[tokio::test]
async fn a_stale_table_names_the_games_patch() {
    let bench = a_bench_with_a_table().await;
    on_patch(&bench.lcu, "16.23");
    assert_eq!(notices_at_start(&bench).await, [STALE_16_23]);
}

#[tokio::test]
async fn an_unknown_patch_has_its_own_line() {
    let bench = a_bench_with_a_table().await;
    bench.lcu.remove(VERSION);
    assert_eq!(notices_at_start(&bench).await, ["Patch unknown"]);
}

#[tokio::test]
async fn an_unknown_role_has_its_own_line() {
    let mut bench = a_bench_with_a_table().await;
    bench.seat = Seat {
        position: "NONE",
        ..MASTER_YI_MID
    };
    assert_eq!(notices_at_start(&bench).await, ["Role unknown"]);
}

#[tokio::test]
async fn an_unknown_tier_and_an_unranked_player_differ() {
    let bench = a_bench_with_a_table().await;
    bench.lcu.remove(RANKED);
    assert_eq!(notices_at_start(&bench).await, ["Tier unknown"]);

    bench.lcu.set(RANKED, &ranked("NONE"));
    assert_eq!(notices_at_start(&bench).await, ["Unranked"]);
}

#[tokio::test]
async fn a_new_season_with_no_schedule_has_its_own_line() {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_serving(&["16.23", "16.24"], None));
    on_patch(&bench.lcu, "17.1");
    assert_eq!(
        notices_at_start(&bench).await,
        ["No schedule for Season 17"]
    );
}

#[tokio::test]
async fn a_thin_cell_has_no_notice() {
    let bench = MarkerBench::new().await;
    bench.write_table(&table_of(&[]));
    assert_eq!(notices_at_start(&bench).await, Vec::<String>::new());
}

#[tokio::test]
async fn a_missing_or_unreadable_table_says_so() {
    let bench = MarkerBench::new().await;
    assert_eq!(notices_at_start(&bench).await, ["No Benchmarks yet"]);

    bench.write_table("not a table");
    assert_eq!(notices_at_start(&bench).await, ["No Benchmarks yet"]);
}

// With no client there is no queue, yet Markers and hints are both gone, so it is said.
#[tokio::test]
async fn an_unavailable_client_is_the_only_line() {
    let mut bench = a_bench_with_a_table().await;
    bench.seat = Seat {
        position: "NONE",
        ..MASTER_YI_MID
    };
    let no_client = TempDir::new().unwrap();
    let url = serve_in_order(vec![Some(body(
        bench.seat,
        Moment::at(1.0),
        &["GameStart"],
    ))])
    .await;
    let clock = GameClock::starting_at(0.0);
    let lockfile = no_client.path().join("lockfile");
    let (_, game) = start_playing(url, &lockfile, &bench.table(), clock)
        .recv()
        .await
        .unwrap();
    assert_eq!(game.unwrap().shown_notices(), ["League client unavailable"]);
}

#[tokio::test]
async fn each_condition_shows_its_own_line_in_order() {
    let mut bench = MarkerBench::new().await;
    bench.write_table(&table_serving(&["16.20", "16.21"], None));
    bench.seat = Seat {
        position: "NONE",
        ..MASTER_YI_MID
    };
    bench.lcu.remove(RANKED);
    on_patch(&bench.lcu, "17.2");
    assert_eq!(
        notices_at_start(&bench).await,
        [
            "Benchmarks out of date (patch 17.2)",
            "Role unknown",
            "Tier unknown",
            "No schedule for Season 17",
        ]
    );
}

#[tokio::test]
async fn a_queue_with_no_markers_or_hints_has_no_notice() {
    let mut bench = MarkerBench::new().await;
    bench.seat = Seat {
        position: "NONE",
        ..MASTER_YI_MID
    };
    bench.lcu.remove(RANKED);
    on_patch(&bench.lcu, "17.2");
    // Swiftplay.
    in_queue(&bench.lcu, 480);
    assert_eq!(notices_at_start(&bench).await, Vec::<String>::new());

    bench.lcu.remove(SESSION);
    assert_eq!(notices_at_start(&bench).await, Vec::<String>::new());
}

// Practice Tool has hints but no Markers, so only what stops a hint is worth a line.
#[tokio::test]
async fn practice_tool_only_notices_what_costs_it_hints() {
    let mut bench = MarkerBench::new().await;
    bench.seat = Seat {
        position: "NONE",
        ..MASTER_YI_MID
    };
    bench.lcu.remove(RANKED);
    in_queue(&bench.lcu, 3140);
    assert_eq!(notices_at_start(&bench).await, Vec::<String>::new());

    on_patch(&bench.lcu, "17.2");
    assert_eq!(
        notices_at_start(&bench).await,
        ["No schedule for Season 17"]
    );

    bench.lcu.remove(VERSION);
    assert_eq!(notices_at_start(&bench).await, ["Patch unknown"]);
}

#[tokio::test]
async fn notices_show_for_the_first_ten_seconds_only() {
    let bench = a_bench_with_a_table().await;
    on_patch(&bench.lcu, "16.23");
    let games = games_at(&bench, &[0.5, 9.9, 10.0, 11.0]).await;
    let shown: Vec<usize> = games.iter().map(|g| g.shown_notices().len()).collect();
    assert_eq!(shown, [1, 1, 0, 0]);
}

// An app restarted mid-game has missed the moment, and a line now would only distract.
#[tokio::test]
async fn joining_a_game_late_shows_no_notice() {
    let bench = a_bench_with_a_table().await;
    on_patch(&bench.lcu, "16.23");
    let games = games_at(&bench, &[60.0]).await;
    assert_eq!(games[0].shown_notices(), Vec::<String>::new());
}
