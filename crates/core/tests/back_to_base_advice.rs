//! Advice on a Back to base, with League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::advice::Trigger;
use lollipop_core::live::ShownAdvice;
use serde_json::{json, Value};

use support::fake_openai::{FakeOpenAi, Reply};
use support::item_game::{item_table, magic_heavy, settle, ItemGame, SETTLING_POLLS};
use support::loading_game::{Updates, ME};

const KILLER: &str = "Foe2#EUW";
const TIP: &str = "Купи Покров банши сейчас: Lux и Zed бьют магией.";
const ROD: (u32, &str) = (1058, "Огромный жезл");
const ROD_COST: f64 = 1200.0;
const WARD: (u32, &str) = (3340, "Тотем-страж");

fn answer(text: Option<&str>, item: Option<u32>) -> String {
    json!({ "advice": text, "item": item }).to_string()
}

/// A game at `game_time` on a magic-heavy enemy team, polled until it runs, with only
/// the Back to base switch on so a death asks nothing of its own.
async fn game_at(game_time: f64, openai: &FakeOpenAi) -> (ItemGame, Updates) {
    let game = ItemGame::running_at(game_time, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_with(&openai.url, |trigger| trigger == Trigger::BackToBase);
    settle(&mut updates, openai).await;
    (game, updates)
}

async fn every_trigger_at(game_time: f64, openai: &FakeOpenAi) -> (ItemGame, Updates) {
    let game = ItemGame::running_at(game_time, &item_table(&magic_heavy())).await;
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, openai).await;
    (game, updates)
}

/// The Widget after buying a Rod at 5:00, which OpenAI answers with `text`.
async fn shown_after_purchase(text: &str) -> (Option<ShownAdvice>, FakeOpenAi) {
    let openai = FakeOpenAi::advising(text).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.buy(&[ROD], ROD_COST);
    (settle(&mut updates, &openai).await, openai)
}

fn inputs(openai: &FakeOpenAi) -> Vec<String> {
    openai
        .bodies()
        .iter()
        .map(|body: &Value| body["input"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn a_purchase_shows_advice_with_the_item_icon() {
    let (shown, _openai) = shown_after_purchase(&answer(Some(TIP), Some(3157))).await;
    let shown = shown.unwrap();

    assert_eq!(shown.trigger, Trigger::BackToBase);
    assert_eq!(shown.text, TIP);
    assert_eq!(
        shown.icon.as_deref(),
        Some("https://ddragon.leagueoflegends.com/cdn/16.21.1/img/item/3157.png")
    );
}

#[tokio::test]
async fn the_prompt_describes_the_visit_without_names() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.give_items(ME, &[(1056, "Кольцо Дорана")]);
    settle(&mut updates, &openai).await;
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 1);
    let input = &inputs[0];
    assert!(input.contains("Language: Russian\n"));
    assert!(input.contains("Moment: the player is back to base"));
    assert!(input.contains("Game time: 5:00\n"));
    assert!(input.contains(
        "Player: Ahri, MIDDLE, level 1, KDA 1/2/3, items Кольцо Дорана, Огромный жезл, gold 50\n"
    ));
    assert!(input.contains("Bought in this visit: Огромный жезл\n"));
    assert_eq!(input.matches("Enemy: ").count(), 5);
    assert!(input.contains("Enemy: Zed, MIDDLE"));
    // The Rod just bought builds into Zhonya's, which takes over from Banshee's minute.
    assert!(input.contains(
        "Item to build: 3157 Песочные часы Жони, bought in 60% of games, first at minute 20\n"
    ));
    for name in ["#EUW", "Foe", "Friend", "Me,"] {
        assert!(!input.contains(name), "{name} is in the prompt");
    }
}

// The 300 gold floor keeps wards, potions and trinket swaps from asking.
#[tokio::test]
async fn a_cheap_purchase_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.buy(&[WARD], 75.0);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn gold_spent_without_a_new_item_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.set_gold(100.0);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

// An undo gives the gold back, so it never reads as a purchase.
#[tokio::test]
async fn an_undo_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.give_items(ME, &[ROD]);
    game.set_gold(2450.0);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

// Starting items are bought before 1:30.
#[tokio::test]
async fn a_purchase_before_one_thirty_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(80.0, &openai).await;
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn a_purchase_while_dead_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.die_to(KILLER, &[], 30.0);
    settle(&mut updates, &openai).await;
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn a_purchase_15_seconds_after_respawn_asks_nothing_and_one_later_asks() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.die_to(KILLER, &[], 30.0);
    settle(&mut updates, &openai).await;
    game.respawn();
    settle(&mut updates, &openai).await;
    game.set_time(315.0);
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;
    assert!(openai.requests().is_empty());

    game.set_time(316.0);
    game.buy(&[(3020, "Сапоги чародея")], 0.0);
    game.buy(&[(1052, "Усиливающий фолиант")], 400.0);
    settle(&mut updates, &openai).await;
    assert_eq!(openai.requests().len(), 1);
}

#[tokio::test]
async fn the_rest_of_a_visit_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.set_gold(5000.0);
    settle(&mut updates, &openai).await;
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;
    game.set_time(330.0);
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;
    assert_eq!(openai.requests().len(), 1);

    game.set_time(331.0);
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;
    assert_eq!(openai.requests().len(), 2);
}

// The model answers null when nothing changed since the last Advice.
#[tokio::test]
async fn null_advice_shows_nothing() {
    let (shown, openai) = shown_after_purchase(&answer(None, None)).await;

    assert_eq!(openai.requests().len(), 1);
    assert_eq!(shown, None);
}

#[tokio::test]
async fn an_answer_over_10_seconds_after_the_purchase_is_dropped() {
    let openai = FakeOpenAi::advising_when_released(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.buy(&[ROD], ROD_COST);
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.set_time(311.0);
    openai.release();

    assert_eq!(settle(&mut updates, &openai).await, None);
}

#[tokio::test]
async fn an_answer_after_a_death_is_dropped() {
    let openai = FakeOpenAi::advising_when_released(&answer(Some(TIP), None)).await;
    let (game, mut updates) = game_at(300.0, &openai).await;
    game.buy(&[ROD], ROD_COST);
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.die_to(KILLER, &[], 2.0);
    for _ in 0..SETTLING_POLLS {
        updates.recv().await.unwrap();
    }
    game.respawn();
    openai.release();

    assert_eq!(settle(&mut updates, &openai).await, None);
}

#[tokio::test]
async fn the_back_to_base_switch_off_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), None)).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_with(&openai.url, |trigger| trigger != Trigger::BackToBase);
    settle(&mut updates, &openai).await;
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn death_advice_replaces_back_to_base_advice() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(Some(TIP), None)),
        Reply::advising(&answer(Some("Не стой у стены против Zed."), None)),
    ])
    .await;
    let (game, mut updates) = every_trigger_at(300.0, &openai).await;
    game.buy(&[ROD], ROD_COST);
    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(shown.trigger, Trigger::BackToBase);

    game.die_to(KILLER, &[], 30.0);

    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(shown.trigger, Trigger::Death);
}

// Death Advice from 5:00 still shows at 5:16, the first moment after respawn that asks.
#[tokio::test]
async fn back_to_base_advice_does_not_replace_death_advice() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(Some("Не стой у стены против Zed."), None)),
        Reply::advising(&answer(Some(TIP), None)),
    ])
    .await;
    let (game, mut updates) = every_trigger_at(300.0, &openai).await;
    game.die_to(KILLER, &[], 2.0);
    settle(&mut updates, &openai).await;
    game.respawn();
    settle(&mut updates, &openai).await;
    game.set_time(316.0);
    game.buy(&[ROD], ROD_COST);

    let shown = settle(&mut updates, &openai).await.unwrap();
    assert_eq!(openai.answered(), 2);
    assert_eq!(shown.trigger, Trigger::Death);
}

// Items count as recommended across Triggers, so the model knows what death Advice said.
#[tokio::test]
async fn an_item_death_advice_showed_goes_in_the_prompt() {
    let openai = FakeOpenAi::advising(&answer(Some(TIP), Some(3102))).await;
    let (game, mut updates) = every_trigger_at(300.0, &openai).await;
    game.die_to(KILLER, &[], 30.0);
    settle(&mut updates, &openai).await;
    game.respawn();
    settle(&mut updates, &openai).await;
    game.set_time(400.0);
    game.buy(&[ROD], ROD_COST);
    settle(&mut updates, &openai).await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 2);
    assert!(inputs[1].contains("Moment: the player is back to base"));
    assert!(inputs[1].contains("Earlier recommended: Покров банши, not bought\n"));
    // The Rod bought builds into Zhonya's, so the inventory outranks Banshee's shown.
    assert!(inputs[1].contains("Item to build: 3157 Песочные часы Жони"));
}
