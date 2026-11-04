//! Why a loading screen's plan did not show, as the log tells it.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use serde_json::json;

use support::fake_openai::FakeOpenAi;
use support::loading_game::{lineup, player, shown_over, LoadingGame, Log, Updates, ME};

const PLAN: &str = "Играй от Electrocute: размени на 3 уровне, пока у Zed нет W.";

fn answer() -> String {
    json!({ "advice": PLAN, "next_item": null, "reason": null, "swap": null, "elixir": null })
        .to_string()
}

fn lines(log: &Log) -> Vec<String> {
    log.lock().unwrap().clone()
}

async fn polls(updates: &mut Updates, count: usize) {
    for _ in 0..count {
        updates.recv().await.unwrap();
    }
}

// Practice Tool gives every player `NONE`, which asks nothing.
#[tokio::test]
async fn a_loading_screen_without_a_lane_says_so() {
    let openai = FakeOpenAi::advising(&answer()).await;
    let players = vec![
        player(ME, "Ahri", "ORDER", "NONE"),
        player("Foe2#EUW", "Zed", "CHAOS", "NONE"),
    ];
    let game = LoadingGame::at_loading_screen(players).await;
    let (mut updates, log) = game.poll_logging(&openai.url, true);
    polls(&mut updates, 3).await;

    assert_eq!(
        lines(&log),
        ["Advice loadingScreen: no lane opponent, not asked"]
    );
}

// The Trigger's switch, the Advice Widget's, a missing key and a refused one all read so.
#[tokio::test]
async fn advice_switched_off_says_so() {
    let openai = FakeOpenAi::advising(&answer()).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let (mut updates, log) = game.poll_logging(&openai.url, false);
    polls(&mut updates, 3).await;

    assert_eq!(
        lines(&log),
        ["Advice loadingScreen: off or no key, not asked"]
    );
}

// Says the plan reached the Widget, which rules out every cause above the view.
#[tokio::test]
async fn a_plan_taken_by_the_widget_says_so() {
    let openai = FakeOpenAi::advising(&answer()).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let (mut updates, log) = game.poll_logging(&openai.url, true);
    while openai.answered() == 0 {
        updates.recv().await.unwrap();
    }
    polls(&mut updates, 3).await;

    assert_eq!(lines(&log), ["Advice loadingScreen: shown"]);
}

#[tokio::test]
async fn a_plan_after_1_30_says_it_was_late() {
    let openai = FakeOpenAi::advising_when_released(&answer()).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let (mut updates, log) = game.poll_logging(&openai.url, true);
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.start_game_at(90.0);
    shown_over(&mut updates, 1).await;
    openai.release();
    shown_over(&mut updates, 3).await;

    assert_eq!(lines(&log), ["Advice loadingScreen: dropped, late"]);
}

// A loading screen League stops answering for drops the call it has in flight.
#[tokio::test]
async fn a_loading_screen_left_mid_call_says_so() {
    let openai = FakeOpenAi::advising_when_released(&answer()).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let (mut updates, log) = game.poll_logging(&openai.url, true);
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.close();
    polls(&mut updates, 10).await;

    assert_eq!(
        lines(&log),
        ["Advice loadingScreen: dropped before its answer"]
    );
}
