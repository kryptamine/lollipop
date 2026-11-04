//! A model suggesting a Swap once the inventory is full, with League and OpenAI faked
//! over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::{BuildItem, LiveUpdate, ShownSwap};
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedReceiver;

use support::fake_openai::{FakeOpenAi, Reply};
use support::item_game::{item_table, magic_heavy, ItemGame, DATA_DRAGON_VERSION, SETTLING_POLLS};
use support::loading_game::ME;

const KILLER: &str = "Foe2#EUW";
const TIP: &str = "Zed ловит тебя у стены: держи E на его прыжок.";
const REASON: &str = "Lux и Caitlyn пробивают тебя: продай Посох пустоты ради Морелономикона.";
const RESPAWN: f64 = 30.0;

const SORCERERS: u32 = 3020;
const STEELCAPS: u32 = 3047;
const BANSHEES: u32 = 3102;
const ZHONYAS: u32 = 3157;
const ARCHANGELS: u32 = 3003;
const RABADONS: u32 = 3089;
const VOID_STAFF: u32 = 3135;
const MORELLO: u32 = 3165;
const ROD: u32 = 1058;

/// The six items of the Build, as the client names them.
const FULL_INVENTORY: [(u32, &str); 6] = [
    (SORCERERS, "Сапоги чародея"),
    (BANSHEES, "Покров банши"),
    (ZHONYAS, "Песочные часы Жони"),
    (ARCHANGELS, "Посох архангела"),
    (RABADONS, "Смертельная шляпа Рабадона"),
    (VOID_STAFF, "Посох пустоты"),
];

type Updates = UnboundedReceiver<LiveUpdate>;

fn answer(swap: &Value) -> String {
    json!({ "advice": TIP, "next_item": null, "reason": null, "swap": swap, "elixir": null })
        .to_string()
}

fn swap(sell: u32, buy: u32, reason: &str) -> Value {
    json!({ "sell": sell, "buy": buy, "reason": reason })
}

/// Polls until OpenAI has answered every request and the poll loop has taken the answers.
async fn settle(updates: &mut Updates, openai: &FakeOpenAi) -> LiveUpdate {
    let mut last = updates.recv().await.unwrap();
    for _ in 1..SETTLING_POLLS {
        last = updates.recv().await.unwrap();
    }
    while openai.answered() < openai.requests().len() {
        last = updates.recv().await.unwrap();
    }
    for _ in 0..3 {
        last = updates.recv().await.unwrap();
    }
    last
}

fn shown(item: u32) -> BuildItem {
    BuildItem {
        item,
        icon: Some(format!(
            "https://ddragon.leagueoflegends.com/cdn/{DATA_DRAGON_VERSION}/img/item/{item}.png"
        )),
        name: None,
    }
}

/// A game at 5:00 on a magic-heavy enemy team, the player holding `items`, after one
/// death to Zed that OpenAI answers with each of `answers` in turn.
async fn after_death(
    items: &[(u32, &str)],
    answers: &[String],
) -> (ItemGame, Updates, FakeOpenAi, LiveUpdate) {
    let replies = answers.iter().map(|text| Reply::advising(text)).collect();
    let openai = FakeOpenAi::replying(replies).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    game.give_items(ME, items);
    let mut updates = game.poll_updates(&openai.url);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    let update = settle(&mut updates, &openai).await;
    (game, updates, openai, update)
}

fn input_of(openai: &FakeOpenAi, request: usize) -> String {
    openai.bodies()[request]["input"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn a_swap_shows_while_the_inventory_is_full() {
    let (_game, _updates, _openai, update) = after_death(
        &FULL_INVENTORY,
        &[answer(&swap(VOID_STAFF, MORELLO, REASON))],
    )
    .await;

    assert!(update.build.is_empty());
    assert_eq!(
        update.swap,
        Some(ShownSwap {
            sell: shown(VOID_STAFF),
            buy: shown(MORELLO),
            reason: REASON.to_string(),
        })
    );
    assert_eq!(update.game.unwrap().advice.unwrap().text, TIP);
}

// Selling frees a slot, which hides the Swap; buying its item ends it for good.
#[tokio::test]
async fn a_swap_clears_once_its_item_is_bought() {
    let (game, mut updates, openai, _update) = after_death(
        &FULL_INVENTORY,
        &[answer(&swap(VOID_STAFF, MORELLO, REASON))],
    )
    .await;
    game.respawn();

    game.sell(VOID_STAFF);
    assert_eq!(settle(&mut updates, &openai).await.swap, None);
    game.buy(&[(MORELLO, "Морелономикон")], 0.0);
    assert_eq!(settle(&mut updates, &openai).await.swap, None);
    game.sell(MORELLO);
    game.buy(&[(VOID_STAFF, "Посох пустоты")], 0.0);
    assert_eq!(settle(&mut updates, &openai).await.swap, None);
}

// A Swap whose item to sell went for another one no longer says what to sell.
#[tokio::test]
async fn a_swap_ends_when_the_inventory_fills_without_its_item_to_sell() {
    let (game, mut updates, openai, _update) = after_death(
        &FULL_INVENTORY,
        &[answer(&swap(VOID_STAFF, MORELLO, REASON))],
    )
    .await;
    game.respawn();

    game.sell(VOID_STAFF);
    game.buy(&[(STEELCAPS, "Бронированные сапоги")], 0.0);
    assert_eq!(settle(&mut updates, &openai).await.swap, None);
    game.sell(STEELCAPS);
    game.buy(&[(VOID_STAFF, "Посох пустоты")], 0.0);
    assert_eq!(settle(&mut updates, &openai).await.swap, None);
}

// One Swap at a time: the one from the later death takes the earlier one's place.
#[tokio::test]
async fn a_newer_swap_replaces_the_one_before() {
    const LATER: &str = "Рабадон уже не нужен: Морелономикон срежет лечение Darius.";
    let answers = [
        answer(&swap(VOID_STAFF, MORELLO, REASON)),
        answer(&swap(RABADONS, MORELLO, LATER)),
    ];
    let (game, mut updates, openai, _update) = after_death(&FULL_INVENTORY, &answers).await;
    game.respawn();
    game.set_time(400.0);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);

    let swap = settle(&mut updates, &openai).await.swap.unwrap();
    assert_eq!(swap.sell, shown(RABADONS));
    assert_eq!(swap.reason, LATER);
}

// No Swap the model names outside what the player holds and the Item candidates
// reaches the Widget: see ADR 0004.
#[tokio::test]
async fn an_invalid_swap_is_dropped() {
    let invalid = [
        ("sells boots", swap(SORCERERS, MORELLO, REASON)),
        ("sells an item not held", swap(MORELLO, MORELLO, REASON)),
        (
            "buys outside the candidates",
            swap(VOID_STAFF, 6692, REASON),
        ),
        ("buys no completed item", swap(VOID_STAFF, ROD, REASON)),
        ("buys an item held", swap(VOID_STAFF, RABADONS, REASON)),
        (
            "buys a second pair of boots",
            swap(VOID_STAFF, STEELCAPS, REASON),
        ),
        ("has a blank reason", swap(VOID_STAFF, MORELLO, " ")),
    ];
    for (what, invalid) in invalid {
        let (_game, _updates, _openai, update) =
            after_death(&FULL_INVENTORY, &[answer(&invalid)]).await;

        assert_eq!(update.swap, None, "a Swap that {what} showed");
        assert_eq!(update.game.unwrap().advice.unwrap().text, TIP);
    }
}

// A reason longer than the Widget holds is never cut short: the whole answer goes.
#[tokio::test]
async fn a_swap_reason_over_120_characters_drops_the_answer() {
    let long = "а".repeat(121);
    let (_game, _updates, _openai, update) = after_death(
        &FULL_INVENTORY,
        &[answer(&swap(VOID_STAFF, MORELLO, &long))],
    )
    .await;

    assert_eq!(update.swap, None);
    assert_eq!(update.game.unwrap().advice, None);
}

// The Build still has room, so it, not a Swap, says what to buy.
#[tokio::test]
async fn no_swap_while_a_slot_is_free() {
    let (_game, _updates, openai, update) = after_death(
        &FULL_INVENTORY[..5],
        &[answer(&swap(ZHONYAS, MORELLO, REASON))],
    )
    .await;

    assert_eq!(update.swap, None);
    assert_eq!(update.build, [shown(VOID_STAFF)]);
    assert!(!input_of(&openai, 0).contains("Swap"));
}

#[tokio::test]
async fn the_death_prompt_offers_what_a_swap_may_sell_and_buy() {
    let (_game, _updates, openai, _update) =
        after_death(&FULL_INVENTORY, &[answer(&Value::Null)]).await;

    let input = input_of(&openai, 0);
    assert!(input.contains("Build: none\n"));
    assert!(input.contains("Inventory full\n"));
    assert!(input.contains("Swap sell option: 3135 Посох пустоты\n"));
    assert!(
        input.contains("Swap sell option: 3157 Песочные часы Жони (SpellDamage, Armor, Active)\n")
    );
    assert_eq!(input.matches("Swap sell option: ").count(), 5);
    assert!(input
        .contains("Swap candidate: 3165 item 3165, bought in 10% of games, first at minute 18\n"));
    assert_eq!(input.matches("Swap candidate: ").count(), 1);
    let instructions = openai.bodies()[0]["instructions"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(instructions.contains("`swap` may sell one Swap sell option"));
}
