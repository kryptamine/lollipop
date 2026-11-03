//! Markers against a Benchmark table on disk, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::sync::Arc;

use lollipop_core::live::Marker::{Above, Below, Neutral};
use lollipop_core::live::{Marker, Markers};
use serde_json::Value;

use support::benchmark_file::{cell, flat_minutes, minute, table_of, with_scores, ENOUGH_GAMES};
use support::fake_lcu::{ranked, RANKED, SESSION, VERSION};
use support::game_clock::GameClock;
use support::marker_game::{Beat, MarkerBench, Moment, Seat, CHAMPION, MASTER_YI_MID, ROLE, TIER};
use support::polling::start_playing;
use support::recorded_game::RecordedGame;
use support::server::serve;

fn minutes_of(benchmark_at: impl Fn(f64) -> (f64, f64, f64)) -> Vec<Value> {
    (0..=40)
        .map(|at| {
            let (cs, gold, level) = benchmark_at(f64::from(at));
            minute(at, cs, gold, level, ENOUGH_GAMES)
        })
        .collect()
}

async fn cs_marker_at(bench: &MarkerBench, cs: u32) -> Option<Marker> {
    bench.markers_for(&[Moment::at(100.0).cs(cs)]).await[0].cs
}

/// Flat at `gold` from minute 1, and at the starting 500 at minute 0, so the poll a gold
/// count needs before 1:00 enters no state of its own.
fn gold_cell(gold: f64) -> String {
    let mut minutes = flat_minutes(50.0, gold, 10.0);
    minutes[0] = minute(0, 50.0, 500.0, 10.0, ENOUGH_GAMES);
    table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)])
}

fn cs_of(markers: &[Markers]) -> Vec<Option<Marker>> {
    markers.iter().map(|m| m.cs).collect()
}

const NO_MARKERS: Markers = Markers {
    cs: None,
    gold: None,
    level: None,
    scores: None,
};

// Protects the whole path on real data. The expected states are worked out by hand from
// the recording's values at each mark, against Benchmarks that grow steadily.
#[tokio::test]
async fn replays_a_recorded_game_against_the_table() {
    let bench = MarkerBench::new().await;
    let minutes = minutes_of(|at| {
        (
            6.0 * at,
            400.0f64.mul_add(at, 500.0),
            0.5f64.mul_add(at, 1.0),
        )
    });
    bench.write_table(&table_of(&[cell("Warwick", "JUNGLE", TIER, &minutes)]));

    let game = Arc::new(RecordedGame::load());
    let clock = GameClock::starting_at(game.first_game_time());
    let server_clock = clock.clone();
    let server_game = Arc::clone(&game);
    let url = serve(Box::new(move || server_game.payload_at(server_clock.now()))).await;
    let mut updates = start_playing(url, &bench.lcu.lockfile(), &bench.table(), clock);

    let mut at_mark = Vec::new();
    let mut next_mark = 5.0;
    let mut in_game = false;
    loop {
        let (game_time, update) = updates.recv().await.unwrap();
        let Some(game) = update else {
            if in_game {
                break;
            }
            continue;
        };
        in_game = true;
        if game_time >= next_mark * 60.0 {
            at_mark.push((game.markers.cs, game.markers.gold, game.markers.level));
            next_mark += 5.0;
        }
    }

    // CS + 5 is 35, 75, 105, 145, 185, 205. Gold earned is 1738, 4125, 6858, 9217, 13098,
    // 15411. Level is 4, 8, 10, 13, 16, 17. Two marks keep a state inside the zone: level 4
    // went 1.35 ahead at 3:18, and at 30:00 level is on the edge of the zone.
    assert_eq!(
        at_mark,
        [
            (Some(Neutral), Some(Below), Some(Above)),
            (Some(Above), Some(Below), Some(Above)),
            (Some(Above), Some(Neutral), Some(Above)),
            (Some(Above), Some(Above), Some(Above)),
            (Some(Above), Some(Above), Some(Above)),
            (Some(Above), Some(Above), Some(Above)),
        ]
    );
}

// Benchmarks at 10:00 and 11:00 are 50 and 70, so 10:30 reads 60. CS 63 compares as 68,
// on the edge of the floor of 8. Any other read of either value would flip one case.
#[tokio::test]
async fn compares_cs_plus_five_against_the_interpolated_benchmark() {
    let bench = MarkerBench::new().await;
    let minutes: Vec<Value> = [(9, 50.0), (10, 50.0), (11, 70.0)]
        .into_iter()
        .map(|(at, cs)| minute(at, cs, 1000.0, 10.0, ENOUGH_GAMES))
        .collect();
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));

    let on_the_edge = [Moment::at(630.0).cs(63)];
    let past_the_edge = [Moment::at(630.0).cs(64)];

    assert_eq!(bench.markers_for(&on_the_edge).await[0].cs, Some(Neutral));
    assert_eq!(bench.markers_for(&past_the_edge).await[0].cs, Some(Above));
}

// Benchmarks at 0:00 and 1:00 give 1000 gold and level 6 at 0:30. Reading either minute
// alone would flip one of these. The poll at 0:00 sits on minute 0 and enters no state.
#[tokio::test]
async fn interpolates_gold_and_level_too() {
    let bench = MarkerBench::new().await;
    let minutes = [
        minute(0, 0.0, 500.0, 1.0, ENOUGH_GAMES),
        minute(1, 0.0, 1500.0, 11.0, ENOUGH_GAMES),
    ];
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));
    let polls = |gold, level| {
        [
            Moment::at(0.0).level(1),
            Moment::at(30.0).gold(gold).level(level),
        ]
    };

    let on_the_edge = bench.markers_for(&polls(1150.0, 7)).await;
    let past_the_edge = bench.markers_for(&polls(1151.0, 8)).await;

    assert_eq!(on_the_edge[1].gold, Some(Neutral));
    assert_eq!(on_the_edge[1].level, Some(Neutral));
    assert_eq!(past_the_edge[1].gold, Some(Above));
    assert_eq!(past_the_edge[1].level, Some(Above));
}

#[tokio::test]
async fn cs_neutral_zone_is_8_percent_with_a_floor_of_8() {
    let cases = [
        (50.0, 53, Neutral),
        (50.0, 54, Above),
        (50.0, 37, Neutral),
        (50.0, 36, Below),
        (200.0, 211, Neutral),
        (200.0, 212, Above),
        (200.0, 179, Neutral),
        (200.0, 178, Below),
    ];
    for (benchmark, cs, expected) in cases {
        let bench = MarkerBench::with_flat_cell(benchmark, 1000.0, 10.0).await;
        let marker = cs_marker_at(&bench, cs).await;
        assert_eq!(marker, Some(expected), "{cs} against {benchmark}");
    }
}

#[tokio::test]
async fn gold_neutral_zone_is_8_percent_with_a_floor_of_150() {
    let cases = [
        (1000.0, 1150.0, Neutral),
        (1000.0, 1151.0, Above),
        (1000.0, 850.0, Neutral),
        (1000.0, 849.0, Below),
        (5000.0, 5400.0, Neutral),
        (5000.0, 5401.0, Above),
        (5000.0, 4600.0, Neutral),
        (5000.0, 4599.0, Below),
    ];
    for (benchmark, earned, expected) in cases {
        let bench = MarkerBench::new().await;
        bench.write_table(&gold_cell(benchmark));
        let moments = [Moment::at(1.0), Moment::at(100.0).gold(earned)];
        let marker = bench.markers_for(&moments).await[1].gold;
        assert_eq!(marker, Some(expected), "{earned} against {benchmark}");
    }
}

#[tokio::test]
async fn level_neutral_zone_is_one_level() {
    let cases = [
        (10.0, 11, Neutral),
        (10.0, 12, Above),
        (10.0, 9, Neutral),
        (10.0, 8, Below),
        (10.5, 12, Above),
        (10.5, 11, Neutral),
    ];
    for (benchmark, level, expected) in cases {
        let bench = MarkerBench::with_flat_cell(50.0, 1000.0, benchmark).await;
        let marker = bench.markers_for(&[Moment::at(100.0).level(level)]).await[0].level;
        assert_eq!(marker, Some(expected), "{level} against {benchmark}");
    }
}

// Gold 2000 against 1000 showed a neutral dot for 30 s: a lead past the zone shows at once.
#[tokio::test]
async fn a_state_shows_on_the_first_poll_past_the_zone() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let moments = [Moment::at(100.0).cs(95), Moment::at(101.0).cs(104)];
    let markers = bench.markers_for(&moments).await;
    assert_eq!(cs_of(&markers), [Some(Neutral), Some(Above)]);
}

// CS + 5 against 100 has a zone of 8. A wave on the edge must not flicker the triangle, so
// it stays until the value is back within 4, and neutral needs the whole 8 to leave.
#[tokio::test]
async fn a_state_stays_until_the_value_is_back_within_half_the_zone() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let moments = [
        Moment::at(100.0).cs(100),
        Moment::at(101.0).cs(104),
        Moment::at(102.0).cs(100),
        Moment::at(103.0).cs(99),
        Moment::at(104.0).cs(100),
    ];
    let markers = bench.markers_for(&moments).await;
    assert_eq!(
        cs_of(&markers),
        [
            Some(Neutral),
            Some(Above),
            Some(Above),
            Some(Neutral),
            Some(Neutral)
        ]
    );
}

#[tokio::test]
async fn crossing_from_above_to_below_shows_at_once() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let moments = [Moment::at(100.0).cs(200), Moment::at(101.0).cs(10)];
    let markers = bench.markers_for(&moments).await;
    assert_eq!(cs_of(&markers), [Some(Above), Some(Below)]);
}

// A state from one game must not keep the next game's Marker coloured.
#[tokio::test]
async fn a_new_game_starts_afresh() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let beats = [
        Beat::Running(Moment::at(100.0).cs(200)),
        Beat::Ended(Moment::at(101.0).cs(200)),
        Beat::Running(Moment::at(1.0).cs(100)),
    ];
    let markers = bench.markers_over(beats.to_vec(), |_| {}).await;
    assert_eq!(markers[0].cs, Some(Above));
    assert_eq!(markers[2].cs, Some(Neutral));
}

/// Flat at 2 kills, 1 death and 3 assists, so kills and assists less deaths is 4.
fn scored_cell(scores_n: u32) -> String {
    let minutes: Vec<Value> = flat_minutes(100.0, 1000.0, 10.0)
        .into_iter()
        .map(|minute| with_scores(minute, [2.0, 1.0, 3.0], scores_n))
        .collect();
    table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)])
}

// One Marker reads kills and assists less deaths against the Cell's, with a zone of 2.
#[tokio::test]
async fn the_scores_marker_compares_kills_and_assists_less_deaths() {
    let cases = [
        ([4, 1, 3], Neutral),
        ([4, 1, 4], Above),
        ([1, 2, 3], Neutral),
        ([1, 3, 3], Below),
        ([0, 0, 7], Above),
    ];
    for ([kills, deaths, assists], expected) in cases {
        let bench = MarkerBench::new().await;
        bench.write_table(&scored_cell(ENOUGH_GAMES));
        let moment = Moment::at(100.0).scores(kills, deaths, assists);
        let marker = bench.markers_for(&[moment]).await[0].scores;
        assert_eq!(marker, Some(expected), "{kills}/{deaths}/{assists}");
    }
}

// Scores come only from games kept since they were counted, so they thin out on their
// own. A thin score must cost the scores Marker alone.
#[tokio::test]
async fn thin_scores_hide_only_the_scores_marker() {
    let bench = MarkerBench::new().await;
    bench.write_table(&scored_cell(99));
    let markers = bench
        .markers_for(&[Moment::at(100.0).scores(9, 0, 9)])
        .await;
    assert_eq!(markers[0].scores, None);
    assert_eq!(markers[0].cs, Some(Neutral));
}

#[tokio::test]
async fn a_table_or_payload_without_scores_has_no_scores_marker() {
    let no_scores_in_table = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let moment = Moment::at(100.0).scores(9, 0, 9);
    assert_eq!(
        no_scores_in_table.markers_for(&[moment]).await[0].scores,
        None
    );

    let no_scores_in_payload = MarkerBench::new().await;
    no_scores_in_payload.write_table(&scored_cell(ENOUGH_GAMES));
    let markers = no_scores_in_payload.markers_for(&[Moment::at(100.0)]).await;
    assert_eq!(markers[0].scores, None);
}

// ADR 0003: a Marker cannot change meaning mid-match, so a new file waits for the next game.
#[tokio::test]
async fn a_table_changed_during_a_game_applies_to_the_next_game() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let table = bench.table();
    let beats = [
        Beat::Running(Moment::at(100.0)),
        Beat::Running(Moment::at(101.0)),
        Beat::Ended(Moment::at(102.0)),
        Beat::Running(Moment::at(1.0)),
    ];
    let markers = bench
        .markers_over(beats.to_vec(), move |poll| {
            if poll == 1 {
                std::fs::write(&table, "not a table").unwrap();
            }
        })
        .await;
    assert_eq!(markers[1].cs, Some(Neutral));
    assert_eq!(markers[3], NO_MARKERS);
}

// The connection can drop before a poll sees GameEnd, and the clock then shows the new game.
#[tokio::test]
async fn a_clock_turned_back_reads_the_table_again() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let table = bench.table();
    let beats = [
        Beat::Running(Moment::at(100.0)),
        Beat::Running(Moment::at(5.0)),
    ];
    let markers = bench
        .markers_over(beats.to_vec(), move |poll| {
            if poll == 1 {
                std::fs::write(&table, "not a table").unwrap();
            }
        })
        .await;
    assert_eq!(markers[0].cs, Some(Neutral));
    assert_eq!(markers[1], NO_MARKERS);
}

// Supports do not farm, so CS against other supports would only ever say noise.
#[tokio::test]
async fn a_support_has_no_cs_marker() {
    let mut bench = MarkerBench::new().await;
    bench.seat = Seat {
        position: "UTILITY",
        ..MASTER_YI_MID
    };
    bench.write_table(&table_of(&[cell(
        CHAMPION,
        "UTILITY",
        TIER,
        &flat_minutes(100.0, 500.0, 10.0),
    )]));

    let markers = bench.markers_for(&[Moment::at(1.0)]).await;

    assert_eq!(markers[0].cs, None);
    assert_eq!(markers[0].gold, Some(Neutral));
    assert_eq!(markers[0].level, Some(Neutral));
}

// Minute 10 is thin. A blend on either side of it would rest on too few games.
#[tokio::test]
async fn a_minute_with_n_below_100_has_no_marker() {
    let bench = MarkerBench::new().await;
    let minutes = [
        minute(9, 100.0, 1000.0, 10.0, 100),
        minute(10, 100.0, 1000.0, 10.0, 99),
        minute(11, 100.0, 1000.0, 10.0, 100),
    ];
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));

    let moments = [
        Moment::at(540.0),
        Moment::at(570.0),
        Moment::at(600.0),
        Moment::at(630.0),
        Moment::at(660.0),
    ];
    let markers = bench.markers_for(&moments).await;

    assert_eq!(
        cs_of(&markers),
        [Some(Neutral), None, None, None, Some(Neutral)]
    );
}

// Without this, a thin stretch would hide a colour and bring back one from before it.
#[tokio::test]
async fn a_marker_comes_back_neutral_after_a_thin_minute() {
    let bench = MarkerBench::new().await;
    let minutes = [
        minute(1, 100.0, 1000.0, 10.0, 100),
        minute(2, 100.0, 1000.0, 10.0, 100),
        minute(3, 100.0, 1000.0, 10.0, 99),
        minute(4, 100.0, 1000.0, 10.0, 100),
        minute(5, 100.0, 1000.0, 10.0, 100),
    ];
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));

    let moments = [
        Moment::at(60.0).cs(200),
        Moment::at(180.0).cs(200),
        Moment::at(240.0).cs(100),
    ];
    let markers = bench.markers_for(&moments).await;

    assert_eq!(cs_of(&markers), [Some(Above), None, Some(Neutral)]);
}

// A game counts only towards the minutes it reached, so a long game runs off the table.
#[tokio::test]
async fn past_the_last_minute_there_is_no_marker() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let moments = [Moment::at(2400.0), Moment::at(2400.5)];
    assert_eq!(
        cs_of(&bench.markers_for(&moments).await),
        [Some(Neutral), None]
    );
}

// Stretching across a hole would present a two-minute guess as a Benchmark.
#[tokio::test]
async fn a_missing_minute_has_no_marker() {
    let bench = MarkerBench::new().await;
    let minutes = [
        minute(1, 100.0, 1000.0, 10.0, 100),
        minute(3, 100.0, 1000.0, 10.0, 100),
    ];
    bench.write_table(&table_of(&[cell(CHAMPION, ROLE, TIER, &minutes)]));
    assert_eq!(bench.markers_for(&[Moment::at(120.0)]).await[0], NO_MARKERS);
}

// A late start has a dash for gold, and that must not take CS and level with it.
#[tokio::test]
async fn a_game_with_no_gold_count_keeps_cs_and_level_markers() {
    let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    let markers = bench.markers_for(&[Moment::at(100.0)]).await;
    assert_eq!(markers[0].cs, Some(Neutral));
    assert_eq!(markers[0].gold, None);
    assert_eq!(markers[0].level, Some(Neutral));
}

#[tokio::test]
async fn markers_show_on_ranked_solo_ranked_flex_draft_and_blind() {
    for queue in [420, 440, 400, 430] {
        let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
        let session = format!(r#"{{ "gameData": {{ "queue": {{ "id": {queue} }} }} }}"#);
        bench.lcu.set(SESSION, &session);
        let markers = bench.markers_for(&[Moment::at(100.0)]).await;
        assert_eq!(markers[0].cs, Some(Neutral), "queue {queue}");
    }
}

// 480 is Swiftplay and 3140 the Practice Tool probed in a real client. A table built from
// ranked solo says nothing about either.
#[tokio::test]
async fn swiftplay_and_practice_tool_have_no_markers() {
    for queue in [480, 3140, 450] {
        let bench = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
        let session = format!(r#"{{ "gameData": {{ "queue": {{ "id": {queue} }} }} }}"#);
        bench.lcu.set(SESSION, &session);
        let markers = bench.markers_for(&[Moment::at(100.0)]).await;
        assert_eq!(markers[0], NO_MARKERS, "queue {queue}");
    }
}

// ADR 0004: whatever is unknown, the app says nothing rather than guess.
#[tokio::test]
async fn an_unknown_input_means_no_markers() {
    let unknown_queue = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    unknown_queue.lcu.remove(SESSION);
    let unknown_patch = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    unknown_patch.lcu.remove(VERSION);
    let unknown_tier = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    unknown_tier.lcu.remove(RANKED);
    let unranked = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    unranked.lcu.set(RANKED, &ranked("NONE"));
    let mut unknown_role = MarkerBench::with_flat_cell(100.0, 1000.0, 10.0).await;
    unknown_role.seat = Seat {
        position: "NONE",
        ..MASTER_YI_MID
    };

    for bench in [
        unknown_queue,
        unknown_patch,
        unknown_tier,
        unranked,
        unknown_role,
    ] {
        assert_eq!(bench.markers_for(&[Moment::at(100.0)]).await[0], NO_MARKERS);
    }
}

#[tokio::test]
async fn no_table_or_a_table_that_does_not_read_means_no_markers() {
    let no_table = MarkerBench::new().await;
    assert_eq!(
        no_table.markers_for(&[Moment::at(100.0)]).await[0],
        NO_MARKERS
    );

    let not_json = MarkerBench::new().await;
    not_json.write_table("not a table");
    assert_eq!(
        not_json.markers_for(&[Moment::at(100.0)]).await[0],
        NO_MARKERS
    );

    let later_schema = MarkerBench::new().await;
    let cells = [cell(
        CHAMPION,
        ROLE,
        TIER,
        &flat_minutes(100.0, 1000.0, 10.0),
    )];
    let body = serde_json::json!({ "schema_version": 3, "cells": cells }).to_string();
    later_schema.write_table(&body);
    assert_eq!(
        later_schema.markers_for(&[Moment::at(100.0)]).await[0],
        NO_MARKERS
    );
}

// The build pools adjacent tiers into a Cell's own entry. A neighbour's entry pools other
// tiers again, so reading it would compare a Gold player two tiers away.
#[tokio::test]
async fn another_tier_role_or_champions_cell_is_not_used() {
    let flat = flat_minutes(100.0, 1000.0, 10.0);
    let cells = [
        cell(CHAMPION, ROLE, "SILVER", &flat),
        cell(CHAMPION, ROLE, "PLATINUM", &flat),
        cell(CHAMPION, "TOP", TIER, &flat),
        cell("Ahri", ROLE, TIER, &flat),
    ];
    let bench = MarkerBench::new().await;
    bench.write_table(&table_of(&cells));
    assert_eq!(bench.markers_for(&[Moment::at(100.0)]).await[0], NO_MARKERS);
}

#[tokio::test]
async fn master_and_above_share_one_cell() {
    for tier in ["MASTER", "GRANDMASTER", "CHALLENGER"] {
        let bench = MarkerBench::new().await;
        let flat = flat_minutes(100.0, 1000.0, 10.0);
        bench.write_table(&table_of(&[cell(CHAMPION, ROLE, "MASTER+", &flat)]));
        bench.lcu.set(RANKED, &ranked(tier));
        let markers = bench.markers_for(&[Moment::at(100.0)]).await;
        assert_eq!(markers[0].cs, Some(Neutral), "{tier}");
    }
}

// Match-V5 names the table's champions, Live Client Data the player's. A name that fails
// to match costs that champion its Markers, in silence.
#[tokio::test]
async fn finds_a_cell_across_the_two_apis_names() {
    let cases = [
        ("MasterYi", "Master Yi", None),
        ("Kaisa", "Kai'Sa", None),
        ("DrMundo", "Dr. Mundo", None),
        ("Leblanc", "LeBlanc", None),
        ("MonkeyKing", "Wukong", None),
        ("Nunu", "Nunu & Willump", None),
        ("Renata", "Renata Glasc", None),
        ("FiddleSticks", "Fiddlesticks", None),
        (
            "MasterYi",
            "Мастер Йи",
            Some("game_character_displayname_MasterYi"),
        ),
        (
            "MonkeyKing",
            "Wukong",
            Some("game_character_displayname_MonkeyKing"),
        ),
    ];
    for (in_table, champion, raw_champion) in cases {
        let mut bench = MarkerBench::new().await;
        bench.seat = Seat {
            champion,
            raw_champion,
            ..MASTER_YI_MID
        };
        let flat = flat_minutes(100.0, 1000.0, 10.0);
        bench.write_table(&table_of(&[cell(in_table, ROLE, TIER, &flat)]));
        let markers = bench.markers_for(&[Moment::at(100.0)]).await;
        assert_eq!(markers[0].cs, Some(Neutral), "{champion} as {in_table}");
    }
}
