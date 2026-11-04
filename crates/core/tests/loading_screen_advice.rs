//! Loading screen Advice, with League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;
use std::sync::{Arc, Mutex};

use lollipop_core::advice::Trigger;
use lollipop_core::live::ShownAdvice;
use serde_json::{json, Value};

use support::fake_lcu::FakeLcu;
use support::fake_openai::FakeOpenAi;
use support::hint_game::no_client;
use support::loading_game::{lineup, player, shown_over, LoadingGame, Updates, ME};

const PLAN: &str = "Играй от Electrocute: размени на 3 уровне, пока у Zed нет W.";
const POLLS_ON_LOADING_SCREEN: usize = 8;
// The failed polls in a row after which a loading screen is left without a game,
// as `held_loading_screen.rs` sets it.
const FAILED_POLLS_TO_LEAVE: usize = 10;

fn answer(advice: &str) -> String {
    json!({ "advice": advice, "next_item": null, "reason": null, "swap": null, "elixir": null })
        .to_string()
}

fn switched_on() -> Arc<Mutex<bool>> {
    Arc::new(Mutex::new(true))
}

/// Polls the loading screen until OpenAI has answered and the poll loop has had time
/// to take the answer, or for a fixed while when no answer is coming.
async fn wait_on_loading_screen(updates: &mut Updates, openai: &FakeOpenAi) {
    for _ in 0..POLLS_ON_LOADING_SCREEN {
        assert!(updates.recv().await.unwrap().is_none());
    }
    while openai.answered() < openai.requests().len() {
        updates.recv().await.unwrap();
    }
    for _ in 0..3 {
        updates.recv().await.unwrap();
    }
}

/// The Widget over the first 25 s of a game whose loading screen OpenAI answered with `text`.
async fn shown_after_answer(text: &str) -> (FakeOpenAi, Vec<Option<ShownAdvice>>) {
    let openai = FakeOpenAi::advising(text).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    game.start_game_at(0.0);
    let shown = shown_over(&mut updates, 25).await;
    (openai, shown)
}

/// The requests OpenAI got over a loading screen of these players.
async fn requests_for(players: Vec<Value>, lockfile: &Path) -> Vec<Value> {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(players).await;
    let mut updates = game.poll(&openai.url, lockfile, switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    openai.bodies()
}

async fn input_for(players: Vec<Value>) -> String {
    let bodies = requests_for(players, &no_client()).await;
    assert_eq!(bodies.len(), 1);
    bodies[0]["input"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn the_lane_plan_shows_for_the_first_20_seconds_of_the_game() {
    let (_openai, shown) = shown_after_answer(&answer(PLAN)).await;

    let advice = shown[0].as_ref().unwrap();
    assert_eq!(advice.trigger, Trigger::LoadingScreen);
    assert_eq!(advice.text, PLAN);
    assert!((advice.time_left - 1.0).abs() < 1e-9);
    assert!(shown[..20].iter().all(Option::is_some));
    assert!(shown[20..].iter().all(Option::is_none));
}

// The bar under the text runs down over the Widget's 20 s.
#[tokio::test]
async fn the_time_left_runs_down() {
    let (_openai, shown) = shown_after_answer(&answer(PLAN)).await;

    let left = shown[10].as_ref().unwrap().time_left;
    assert!((left - 0.5).abs() < 1e-9);
}

#[tokio::test]
async fn the_request_has_the_fixed_shape() {
    let lcu = FakeLcu::start().await;
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &lcu.lockfile(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;

    let requests = openai.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].starts_with("post /v1/responses "));
    assert!(requests[0].contains("authorization: bearer sk-proj-fake0123abcd\r\n"));
    let body = &openai.bodies()[0];
    assert_eq!(body["model"], "gpt-6-luna");
    assert_eq!(body["store"], false);
    assert_eq!(body["reasoning"], json!({ "effort": "none" }));
    assert_eq!(body["max_output_tokens"], 360);
    let format = &body["text"]["format"];
    assert_eq!(format["type"], "json_schema");
    assert_eq!(format["strict"], true);
    assert_eq!(
        format["schema"]["required"],
        json!(["advice", "next_item", "reason", "swap", "elixir"])
    );
    assert_eq!(format["schema"]["additionalProperties"], false);
}

#[tokio::test]
async fn the_prompt_has_the_language_tier_and_lane_opponent() {
    let lcu = FakeLcu::start().await;
    let bodies = requests_for(lineup("Ahri", "MIDDLE"), &lcu.lockfile()).await;
    let input = bodies[0]["input"].as_str().unwrap();

    assert!(input.contains("Language: Russian"));
    assert!(input.contains("Player tier: Gold"));
    assert!(input.contains("Player: Ahri, MIDDLE"));
    assert!(input.contains("Lane opponent: Zed\n"));
    assert!(input.contains("Electrocute, Domination and Sorcery"));
    assert!(input.contains("StatModAdaptive"));
    assert!(input.contains("Enemy: Zed, MIDDLE, keystone Conqueror (Precision, Resolve)"));
    assert!(input.contains("spells Flash, Ignite"));
    assert_eq!(input.matches("Ally: ").count(), 4);
    assert_eq!(input.matches("Enemy: ").count(), 5);
}

// Riot IDs and summoner names never leave the machine.
#[tokio::test]
async fn the_prompt_names_no_player() {
    let input = input_for(lineup("Ahri", "MIDDLE")).await;

    for name in ["#EUW", "Friend", "Foe", "Me,", ME] {
        assert!(!input.contains(name), "{name} is in the prompt");
    }
}

#[tokio::test]
async fn the_bottom_lane_faces_the_enemy_pair_beside_its_partner() {
    let input = input_for(lineup("Jinx", "BOTTOM")).await;

    assert!(input.contains("Lane opponent: Caitlyn and Lux\n"));
    assert!(input.contains("Lane partner: Thresh\n"));
}

#[tokio::test]
async fn a_support_also_faces_the_enemy_pair() {
    let input = input_for(lineup("Thresh", "UTILITY")).await;

    assert!(input.contains("Lane opponent: Caitlyn and Lux\n"));
    assert!(input.contains("Lane partner: Jinx\n"));
}

#[tokio::test]
async fn a_jungler_faces_the_enemy_jungler() {
    let input = input_for(lineup("Vi", "JUNGLE")).await;

    assert!(input.contains("Lane opponent: LeeSin\n"));
    assert!(!input.contains("Lane partner"));
}

// Practice Tool gives every player `NONE`.
#[tokio::test]
async fn no_position_asks_nothing() {
    let players = vec![
        player(ME, "Ahri", "ORDER", "NONE"),
        player("Foe2#EUW", "Zed", "CHAOS", "NONE"),
    ];
    assert!(requests_for(players, &no_client()).await.is_empty());
}

#[tokio::test]
async fn no_enemy_in_the_position_asks_nothing() {
    let mut players = lineup("Ahri", "MIDDLE");
    players.retain(|p| p["championName"] != "Zed");
    assert!(requests_for(players, &no_client()).await.is_empty());
}

#[tokio::test]
async fn the_loading_screen_asks_once() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    wait_on_loading_screen(&mut updates, &openai).await;

    assert_eq!(openai.requests().len(), 1);
}

#[tokio::test]
async fn the_next_game_asks_again() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    game.start_game_at(0.0);
    shown_over(&mut updates, 3).await;
    game.end_game();
    shown_over(&mut updates, 0).await;
    for _ in 0..3 {
        updates.recv().await.unwrap();
    }
    game.back_to_loading_screen();
    wait_on_loading_screen(&mut updates, &openai).await;

    assert_eq!(openai.requests().len(), 2);
}

// A loading screen that closes without a game must not keep the next one from asking.
#[tokio::test]
async fn a_loading_screen_closed_without_a_game_lets_the_next_one_ask() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    game.close();
    silent_for(&mut updates, FAILED_POLLS_TO_LEAVE).await;
    game.back_to_loading_screen();
    wait_on_loading_screen(&mut updates, &openai).await;

    assert_eq!(openai.requests().len(), 2);
}

/// Polls while League answers nothing, as when a poll times out.
async fn silent_for(updates: &mut Updates, polls: usize) {
    for _ in 0..polls {
        assert!(updates.recv().await.unwrap().is_none());
    }
}

// The plan still shows when a poll times out as the game takes over from its
// loading screen.
#[tokio::test]
async fn the_plan_outlasts_a_few_failed_polls_before_the_game() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    game.close();
    silent_for(&mut updates, 3).await;
    game.start_game_at(0.0);
    let shown = shown_over(&mut updates, 1).await;

    assert_eq!(shown[0].as_ref().unwrap().text, PLAN);
    assert_eq!(openai.requests().len(), 1);
}

// A failed poll on the loading screen neither aborts the call in flight nor asks again.
#[tokio::test]
async fn a_failed_poll_keeps_the_question_in_flight() {
    let openai = FakeOpenAi::advising_when_released(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.close();
    silent_for(&mut updates, FAILED_POLLS_TO_LEAVE - 1).await;
    game.back_to_loading_screen();
    openai.release();
    wait_on_loading_screen(&mut updates, &openai).await;
    game.start_game_at(0.0);
    let shown = shown_over(&mut updates, 1).await;

    assert_eq!(shown[0].as_ref().unwrap().text, PLAN);
    assert_eq!(openai.requests().len(), 1);
}

// The first answer of a loading screen can come before the player is listed.
#[tokio::test]
async fn the_loading_screen_waits_for_the_active_player() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let mut players = lineup("Ahri", "MIDDLE");
    players.retain(|p| p["riotId"] != ME);
    let game = LoadingGame::at_loading_screen(players).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    wait_on_loading_screen(&mut updates, &openai).await;
    assert!(openai.requests().is_empty());

    game.set_players(lineup("Ahri", "MIDDLE"));
    wait_on_loading_screen(&mut updates, &openai).await;
    assert_eq!(openai.requests().len(), 1);
}

// With no key, the Advice Widget off or the Trigger off, OpenAI is never called.
#[tokio::test]
async fn advice_switched_off_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), Arc::new(Mutex::new(false)));
    wait_on_loading_screen(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn an_app_started_mid_game_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(PLAN)).await;
    let game = LoadingGame::running_at(300.0, lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    let shown = shown_over(&mut updates, POLLS_ON_LOADING_SCREEN).await;

    assert!(shown.iter().all(Option::is_none));
    assert!(openai.requests().is_empty());
}

/// The Widget after OpenAI answers late, released once the game reaches `released_at`.
async fn shown_when_answered_at(released_at: f64) -> Vec<Option<ShownAdvice>> {
    let openai = FakeOpenAi::advising_when_released(&answer(PLAN)).await;
    let game = LoadingGame::at_loading_screen(lineup("Ahri", "MIDDLE")).await;
    let mut updates = game.poll(&openai.url, &no_client(), switched_on());
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.start_game_at(released_at);
    shown_over(&mut updates, 1).await;
    openai.release();
    shown_over(&mut updates, 25).await
}

// An answer that arrives once the game is under way still gets its 20 s.
#[tokio::test]
async fn a_late_answer_before_1_30_is_shown_from_its_arrival() {
    let shown = shown_when_answered_at(60.0).await;

    let first = shown.iter().position(Option::is_some).unwrap();
    assert!((shown[first].as_ref().unwrap().time_left - 1.0).abs() < 1e-9);
    assert_eq!(shown.iter().filter(|advice| advice.is_some()).count(), 20);
}

#[tokio::test]
async fn an_answer_after_1_30_is_dropped() {
    let shown = shown_when_answered_at(90.0).await;

    assert!(shown.iter().all(Option::is_none));
}

#[tokio::test]
async fn advice_over_120_characters_is_dropped_whole() {
    let (openai, shown) = shown_after_answer(&answer(&"а".repeat(121))).await;

    assert_eq!(openai.answered(), 1);
    assert!(shown.iter().all(Option::is_none));
}

#[tokio::test]
async fn advice_of_exactly_120_characters_is_shown() {
    let (_openai, shown) = shown_after_answer(&answer(&"а".repeat(120))).await;

    assert!(shown[0].is_some());
}

#[tokio::test]
async fn null_advice_shows_nothing() {
    let text =
        json!({ "advice": null, "next_item": null, "reason": null, "swap": null, "elixir": null })
            .to_string();
    let (_openai, shown) = shown_after_answer(&text).await;

    assert!(shown.iter().all(Option::is_none));
}

#[tokio::test]
async fn an_answer_outside_the_schema_is_dropped() {
    let outside = [
        json!({}).to_string(),
        json!({ "advice": PLAN }).to_string(),
        json!({ "advice": PLAN, "next_item": null, "reason": null }).to_string(),
        json!({ "advice": PLAN, "next_item": null, "reason": null, "swap": null }).to_string(),
        json!({ "advice": PLAN, "next_item": null, "reason": null, "swap": null, "elixir": "iron" })
            .to_string(),
        json!({ "advice": PLAN, "next_item": null, "reason": null, "swap": { "sell": 3157 }, "elixir": null })
            .to_string(),
        json!({ "advice": PLAN, "item": 3089 }).to_string(),
        json!({ "advice": PLAN, "why": "spikes" }).to_string(),
        json!({ "advice": 42, "next_item": null, "reason": null, "swap": null, "elixir": null }).to_string(),
        format!("Sure! {}", answer(PLAN)),
    ];
    for text in outside {
        let (_openai, shown) = shown_after_answer(&text).await;
        assert!(shown.iter().all(Option::is_none), "{text} was shown");
    }
}
