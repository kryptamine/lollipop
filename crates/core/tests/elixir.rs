//! The Elixir beside the Baron and Elder Timing hints once the inventory is full, with
//! League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::{BuildItem, LiveUpdate};
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedReceiver;

use support::fake_openai::{FakeOpenAi, Reply};
use support::item_game::{item_table, ItemGame, DATA_DRAGON_VERSION, SETTLING_POLLS};
use support::loading_game::ME;

const KILLER: &str = "Foe2#EUW";
const TIP: &str = "Zed ловит тебя у стены: держи E на его прыжок.";
const RESPAWN: f64 = 30.0;
// Baron spawns at 20:00, so its hint shows `Baron 0:15`.
const BEFORE_BARON: f64 = 1185.0;

const IRON: u32 = 2138;
const SORCERY: u32 = 2139;
const WRATH: u32 = 2140;

/// The six items of the Build, as the client names them.
const FULL_INVENTORY: [(u32, &str); 6] = [
    (3020, "Сапоги чародея"),
    (3102, "Покров банши"),
    (3157, "Песочные часы Жони"),
    (3003, "Посох архангела"),
    (3089, "Смертельная шляпа Рабадона"),
    (3135, "Посох пустоты"),
];

type Updates = UnboundedReceiver<LiveUpdate>;

/// A magic-heavy enemy team, and Ahri dealing `ahri` of her damage as magic, or no
/// share for her at all.
fn shares(ahri: Option<f64>) -> Value {
    let mut shares =
        json!({ "Darius": 0.1, "LeeSin": 0.2, "Zed": 0.1, "Caitlyn": 0.3, "Lux": 0.9 });
    if let Some(ahri) = ahri {
        shares["Ahri"] = json!(ahri);
    }
    shares
}

fn answer(elixir: Option<u32>) -> String {
    json!({ "advice": TIP, "next_item": null, "reason": null, "swap": null, "elixir": elixir })
        .to_string()
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

/// Polls until OpenAI has answered every request and the poll loop has taken the answers.
async fn settle(updates: &mut Updates, openai: Option<&FakeOpenAi>) -> LiveUpdate {
    let mut last = updates.recv().await.unwrap();
    for _ in 1..SETTLING_POLLS {
        last = updates.recv().await.unwrap();
    }
    while openai.is_some_and(|openai| openai.answered() < openai.requests().len()) {
        last = updates.recv().await.unwrap();
    }
    for _ in 0..3 {
        last = updates.recv().await.unwrap();
    }
    last
}

/// Each Timing hint showing, and the Elixir beside it.
fn hints_of(update: &LiveUpdate) -> Vec<(&'static str, Option<BuildItem>)> {
    update
        .game
        .as_ref()
        .unwrap()
        .hints
        .iter()
        .map(|hint| (hint.objective, hint.elixir.clone()))
        .collect()
}

/// A game with no OpenAI key at `game_time`, the player holding `items`.
async fn without_key(
    ahri: Option<f64>,
    items: &[(u32, &str)],
    game_time: f64,
) -> (ItemGame, Updates, LiveUpdate) {
    let game = ItemGame::running_at(game_time, &item_table(&shares(ahri))).await;
    game.give_items(ME, items);
    let mut updates = game.poll_updates_without_key();
    let update = settle(&mut updates, None).await;
    (game, updates, update)
}

/// A game where the player holding `items` dies at 15:00 to Zed, OpenAI answering
/// with each of `answers` in turn, then respawns and waits for Baron.
async fn after_death(
    ahri: Option<f64>,
    items: &[(u32, &str)],
    answers: &[String],
) -> (ItemGame, Updates, FakeOpenAi, LiveUpdate) {
    let replies = answers.iter().map(|text| Reply::advising(text)).collect();
    let openai = FakeOpenAi::replying(replies).await;
    let game = ItemGame::running_at(900.0, &item_table(&shares(ahri))).await;
    game.give_items(ME, items);
    let mut updates = game.poll_updates(&openai.url);
    settle(&mut updates, Some(&openai)).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, Some(&openai)).await;
    game.respawn();
    game.set_time(BEFORE_BARON);
    let update = settle(&mut updates, Some(&openai)).await;
    (game, updates, openai, update)
}

fn input_of(openai: &FakeOpenAi, request: usize) -> String {
    openai.bodies()[request]["input"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
async fn code_picks_sorcery_for_a_magic_champion() {
    let (_game, _updates, update) = without_key(Some(0.9), &FULL_INVENTORY, BEFORE_BARON).await;

    assert_eq!(hints_of(&update), [("Baron", Some(shown(SORCERY)))]);
}

// Iron is the model's alone: code never guesses a champion wants to tank.
#[tokio::test]
async fn code_picks_wrath_for_a_physical_champion() {
    let (_game, _updates, update) = without_key(Some(0.2), &FULL_INVENTORY, BEFORE_BARON).await;

    assert_eq!(hints_of(&update), [("Baron", Some(shown(WRATH)))]);
}

// A wrong Elixir is worse than none: with no damage profile for the champion, code
// has nothing to pick by.
#[tokio::test]
async fn no_elixir_without_the_champions_magic_share() {
    let (_game, _updates, update) = without_key(None, &FULL_INVENTORY, BEFORE_BARON).await;

    assert_eq!(hints_of(&update), [("Baron", None)]);
}

#[tokio::test]
async fn no_elixir_below_500_gold() {
    let game = ItemGame::running_at(BEFORE_BARON, &item_table(&shares(Some(0.9)))).await;
    game.give_items(ME, &FULL_INVENTORY);
    game.set_gold(499.0);
    let mut updates = game.poll_updates_without_key();

    assert_eq!(
        hints_of(&settle(&mut updates, None).await),
        [("Baron", None)]
    );
    game.set_gold(500.0);
    assert_eq!(
        hints_of(&settle(&mut updates, None).await),
        [("Baron", Some(shown(SORCERY)))]
    );
}

// While a slot is free, an item still beats an Elixir.
#[tokio::test]
async fn no_elixir_with_a_free_slot() {
    let (_game, _updates, update) =
        without_key(Some(0.9), &FULL_INVENTORY[..5], BEFORE_BARON).await;

    assert_eq!(hints_of(&update), [("Baron", None)]);
}

#[tokio::test]
async fn the_elixir_shows_with_elder_but_not_with_a_drake() {
    let game = ItemGame::running_at(1500.0, &item_table(&shares(Some(0.9)))).await;
    game.give_items(ME, &FULL_INVENTORY);
    // Three drakes to the enemy team: the next one, at 25:30, is a drake.
    for at in [600.0, 1000.0, 1230.0] {
        game.log_event(json!({ "EventName": "DragonKill", "EventTime": at, "KillerName": KILLER, "DragonType": "Air" }));
    }
    let mut updates = game.poll_updates_without_key();
    assert_eq!(
        hints_of(&settle(&mut updates, None).await),
        [("Drake", None)]
    );

    // Their fourth brings Elder 6:00 later, at 32:00.
    game.log_event(json!({ "EventName": "DragonKill", "EventTime": 1560.0, "KillerName": KILLER, "DragonType": "Air" }));
    game.set_time(1900.0);
    assert_eq!(
        hints_of(&settle(&mut updates, None).await),
        [("Elder", Some(shown(SORCERY)))]
    );
}

// The model may see what code cannot, such as an enemy team that bursts the player.
#[tokio::test]
async fn the_models_pick_wins_over_code() {
    let (_game, _updates, openai, update) =
        after_death(Some(0.9), &FULL_INVENTORY, &[answer(Some(IRON))]).await;

    assert_eq!(hints_of(&update), [("Baron", Some(shown(IRON)))]);
    assert_eq!(openai.requests().len(), 1);
}

// See ADR 0004: a model's answer outside what it was offered never reaches the overlay.
#[tokio::test]
async fn a_pick_that_is_no_elixir_falls_back_to_code() {
    for invalid in [3089, 2003] {
        let (_game, _updates, _openai, update) =
            after_death(Some(0.9), &FULL_INVENTORY, &[answer(Some(invalid))]).await;

        assert_eq!(
            hints_of(&update),
            [("Baron", Some(shown(SORCERY)))],
            "a pick of {invalid} showed"
        );
    }
}

// The pick is asked once the inventory is full; one made with a slot free was never
// offered, so filling the inventory later still shows code's pick.
#[tokio::test]
async fn a_pick_with_a_free_slot_is_ignored() {
    let (game, mut updates, openai, update) =
        after_death(Some(0.9), &FULL_INVENTORY[..5], &[answer(Some(IRON))]).await;
    assert!(!input_of(&openai, 0).contains("Elixir"));
    assert_eq!(hints_of(&update), [("Baron", None)]);

    game.buy(&[FULL_INVENTORY[5]], 0.0);
    assert_eq!(
        hints_of(&settle(&mut updates, Some(&openai)).await),
        [("Baron", Some(shown(SORCERY)))]
    );
}

#[tokio::test]
async fn the_death_prompt_offers_the_elixirs_until_one_is_picked() {
    let answers = [answer(Some(WRATH)), answer(None)];
    let (game, mut updates, openai, _update) =
        after_death(Some(0.9), &FULL_INVENTORY, &answers).await;

    let first = input_of(&openai, 0);
    assert!(first.contains(
        "Elixir options: 2138 Elixir of Iron, 2139 Elixir of Sorcery, 2140 Elixir of Wrath\n"
    ));
    let instructions = openai.bodies()[0]["instructions"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(instructions.contains("`elixir` is the id of the one"));

    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, Some(&openai)).await;
    assert!(!input_of(&openai, 1).contains("Elixir"));
}
