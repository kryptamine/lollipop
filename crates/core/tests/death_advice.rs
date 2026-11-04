//! Advice on death, with League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::time::Duration;

use lollipop_core::advice::Trigger;
use lollipop_core::live::ShownAdvice;
use serde_json::{json, Value};

use support::fake_openai::{FakeOpenAi, Reply};
use support::item_game::{item_table, magic_heavy, settle, ItemGame, SETTLING_POLLS};
use support::loading_game::{Updates, ME};

const KILLER: &str = "Foe2#EUW";
const ASSISTER: &str = "Foe1#EUW";
const TIP: &str = "Zed ловит тебя у стены: держи E на его прыжок.";
const RESPAWN: f64 = 30.0;

fn answer(text: &str) -> String {
    json!({ "advice": text, "next_item": null, "reason": null, "swap": null, "elixir": null })
        .to_string()
}

/// A game at 5:00 on a magic-heavy enemy team, polled until it runs.
async fn game_at_five_minutes(table: &Value, openai: &FakeOpenAi) -> (ItemGame, Updates) {
    let game = ItemGame::running_at(300.0, table).await;
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, openai).await;
    (game, updates)
}

/// The Widget after one death to Zed that OpenAI answers with `text`.
async fn shown_after_death(text: &str) -> Option<ShownAdvice> {
    let openai = FakeOpenAi::advising(text).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[ASSISTER], RESPAWN);
    settle(&mut updates, &openai).await
}

/// The input of the request one death to Zed sends, on a game set up by `prepare`.
async fn input_of_death(table: &Value, prepare: impl FnOnce(&ItemGame)) -> String {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(table, &openai).await;
    prepare(&game);
    game.die_to(KILLER, &[ASSISTER], RESPAWN);
    settle(&mut updates, &openai).await;
    let bodies = openai.bodies();
    assert_eq!(bodies.len(), 1);
    bodies[0]["input"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_death_to_an_enemy_champion_shows_advice() {
    let shown = shown_after_death(&answer(TIP)).await.unwrap();

    assert_eq!(shown.trigger, Trigger::Death);
    assert_eq!(shown.text, TIP);
}

// The Build Widget names the items now, so Advice naming one would contradict it.
#[tokio::test]
async fn death_advice_is_asked_without_items_to_name() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    let body = &openai.bodies()[0];
    assert!(body["instructions"]
        .as_str()
        .unwrap()
        .contains("Never name an item to buy"));
    let input = body["input"].as_str().unwrap();
    assert!(!input.contains("Item option"));
    assert!(!input.contains("Earlier recommended"));
}

// An answer naming an item is outside the schema and is dropped whole.
#[tokio::test]
async fn an_answer_with_an_item_shows_nothing() {
    let with_item = json!({ "advice": TIP, "item": 3102 }).to_string();

    assert_eq!(shown_after_death(&with_item).await, None);
}

// A death that teaches nothing is answered null, and a weak idea never fills the Widget.
#[tokio::test]
async fn null_advice_shows_nothing() {
    let null =
        json!({ "advice": null, "next_item": null, "reason": null, "swap": null, "elixir": null })
            .to_string();

    assert_eq!(shown_after_death(&null).await, None);
}

// Every Trigger may stay silent, and generic tips are named so the model avoids them.
#[tokio::test]
async fn the_instructions_prefer_silence_to_a_generic_tip() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    let instructions = openai.bodies()[0]["instructions"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(instructions.contains("Never a tip that fits any game"));
    assert!(instructions.contains("silence beats a weak idea"));
    assert!(instructions.contains("When the death teaches nothing"));
}

#[tokio::test]
async fn the_prompt_describes_the_death_without_names() {
    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(ME, &[(1056, "Кольцо Дорана")]);
        game.give_items(KILLER, &[(6692, "Затмение")]);
    })
    .await;

    assert!(input.contains("Language: Russian\n"));
    assert!(input.contains("Game time: 5:00\n"));
    assert!(input
        .contains("Player: Ahri, MIDDLE, level 1, KDA 1/2/3, items Кольцо Дорана, gold 1250\n"));
    assert!(input
        .contains("Killer: Zed, MIDDLE, level 1, KDA 5/1/0, items Затмение, magic damage 10%\n"));
    assert!(input.contains("Assister: LeeSin, JUNGLE"));
    assert!(input.contains("Deaths to this killer this game: 1\n"));
    assert_eq!(input.matches("Enemy: ").count(), 5);
    for name in ["#EUW", "Foe", "Friend", "Me,"] {
        assert!(!input.contains(name), "{name} is in the prompt");
    }
}

// The model reads who is ahead from the score; an execution would inflate it.
#[tokio::test]
async fn the_prompt_carries_the_kill_score_without_executions() {
    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.log_event(json!({
            "EventName": "ChampionKill",
            "EventTime": 200.0,
            "KillerName": "Friend1#EUW",
            "VictimName": ASSISTER,
            "Assisters": [],
        }));
        game.log_event(json!({
            "EventName": "ChampionKill",
            "EventTime": 250.0,
            "KillerName": "Turret_T2_C_05_A",
            "VictimName": "Friend1#EUW",
            "Assisters": [],
        }));
    })
    .await;

    assert!(input.contains("Kills: player's team 1, enemy team 1\n"));
}

#[tokio::test]
async fn deaths_to_the_same_killer_are_counted() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    game.respawn();
    game.set_time(400.0);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    let bodies = openai.bodies();
    assert_eq!(bodies.len(), 2);
    assert!(bodies[1]["input"]
        .as_str()
        .unwrap()
        .contains("Deaths to this killer this game: 2\n"));
}

// A lone death to an even opponent teaches little; dying to them again is a pattern.
#[tokio::test]
async fn an_even_killer_asks_only_on_the_second_death() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(ASSISTER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    assert!(openai.requests().is_empty());

    game.respawn();
    game.set_time(400.0);
    settle(&mut updates, &openai).await;
    game.die_to(ASSISTER, &[], RESPAWN);
    let shown = settle(&mut updates, &openai).await;

    assert_eq!(openai.requests().len(), 1);
    assert_eq!(shown.unwrap().text, TIP);
}

// A turret, a minion or a monster gets no Advice, and neither does an ally's death.
#[tokio::test]
async fn other_kills_ask_nothing() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to("Turret_T2_C_05_A", &[], RESPAWN);
    game.log_event(json!({
        "EventName": "ChampionKill",
        "EventTime": 300.0,
        "KillerName": KILLER,
        "VictimName": "Friend0#EUW",
        "Assisters": [],
    }));
    let shown = settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
    assert_eq!(shown, None);
}

#[tokio::test]
async fn the_death_switch_off_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_with(&openai.url, |trigger| trigger != Trigger::Death);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

// An app started while the player is dead does not advise on that death.
#[tokio::test]
async fn a_death_before_the_first_poll_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    game.die_to(KILLER, &[], RESPAWN);
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn an_answer_after_respawn_is_dropped() {
    let openai = FakeOpenAi::advising_when_released(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.respawn();
    openai.release();

    assert_eq!(settle(&mut updates, &openai).await, None);
}

// Once shown, death Advice keeps the Widget's 20 s even after the player respawns.
#[tokio::test]
async fn the_advice_stays_its_20_seconds_after_respawn() {
    let openai = FakeOpenAi::advising(&answer(TIP)).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    assert!(settle(&mut updates, &openai).await.is_some());

    game.respawn();
    game.set_time(319.0);
    assert!(settle(&mut updates, &openai).await.is_some());
    game.set_time(320.0);
    assert_eq!(settle(&mut updates, &openai).await, None);
}

#[tokio::test]
async fn a_new_death_replaces_the_advice_showing() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer("Первая смерть.")),
        Reply::advising(&answer("Вторая смерть.")),
    ])
    .await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], 2.0);
    assert_eq!(
        settle(&mut updates, &openai).await.unwrap().text,
        "Первая смерть."
    );

    game.respawn();
    game.set_time(310.0);
    game.die_to(KILLER, &[], RESPAWN);

    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(shown.text, "Вторая смерть.");
    assert!((shown.time_left - 1.0).abs() < 1e-9);
}

const LANE_PLAN: &str = "Играй от Electrocute.";

#[tokio::test]
async fn death_advice_replaces_the_loading_screen_advice() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(LANE_PLAN)),
        Reply::advising(&answer(TIP)),
    ])
    .await;
    let game = ItemGame::at_loading_screen(&item_table(&magic_heavy())).await;
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, &openai).await;
    game.start_game();
    game.set_time(5.0);
    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(shown.trigger, Trigger::LoadingScreen);

    game.die_to(KILLER, &[], RESPAWN);

    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(shown.trigger, Trigger::Death);
}

// Rank is checked as an answer arrives: a lane plan answered late gives way.
#[tokio::test]
async fn a_late_loading_screen_answer_does_not_replace_death_advice() {
    let openai = FakeOpenAi::replying(vec![
        Reply::Delayed {
            wait: Duration::from_millis(500),
            reply: Box::new(Reply::advising(&answer(LANE_PLAN))),
        },
        Reply::advising(&answer(TIP)),
    ])
    .await;
    let game = ItemGame::at_loading_screen(&item_table(&magic_heavy())).await;
    let mut updates = game.poll(&openai.url);
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.start_game();
    game.set_time(5.0);
    for _ in 0..SETTLING_POLLS {
        updates.recv().await.unwrap();
    }
    game.die_to(KILLER, &[], RESPAWN);

    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(openai.answered(), 2);
    assert_eq!(shown.trigger, Trigger::Death);
    assert_eq!(shown.text, TIP);
}
