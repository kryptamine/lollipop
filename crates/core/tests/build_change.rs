//! A model replacing the Next item, with League and OpenAI faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::advice::Trigger;
use lollipop_core::live::LiveUpdate;
use lollipop_core::voiceover::{Reader, Speech};
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedReceiver;

use support::fake_openai::FakeOpenAi;
use support::item_game::{item_table, magic_heavy, with_threats, ItemGame, SETTLING_POLLS};
use support::loading_game::ME;

const KILLER: &str = "Foe2#EUW";
const TIP: &str = "Zed ловит тебя у стены: держи E на его прыжок.";
const REASON: &str = "Zed и Lee Sin бьют физически: сначала бронированные сапоги.";
const RESPAWN: f64 = 30.0;

const SORCERERS: (u32, &str) = (3020, "Сапоги чародея");
const BLASTING_WAND: (u32, &str) = (1026, "Взрывной жезл");
const STEELCAPS: u32 = 3047;
const RABADONS: u32 = 3089;
const ZHONYAS: u32 = 3157;

const FULL_BUILD: [u32; 6] = [3020, 3102, 3157, 3003, 3089, 3135];

type Updates = UnboundedReceiver<LiveUpdate>;

fn answer(next_item: Option<u32>, reason: Option<&str>) -> String {
    json!({ "advice": TIP, "next_item": next_item, "reason": reason, "swap": null, "elixir": null })
        .to_string()
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

fn items(update: &LiveUpdate) -> Vec<u32> {
    update.build.iter().map(|shown| shown.item).collect()
}

/// A game at 5:00 on a magic-heavy enemy team, set up by `prepare`, after one death to
/// Zed that OpenAI answers with `text`.
async fn after_death(text: &str, prepare: impl FnOnce(&ItemGame)) -> (LiveUpdate, FakeOpenAi) {
    after_death_on(&item_table(&magic_heavy()), text, prepare).await
}

async fn after_death_on(
    table: &Value,
    text: &str,
    prepare: impl FnOnce(&ItemGame),
) -> (LiveUpdate, FakeOpenAi) {
    let openai = FakeOpenAi::advising(text).await;
    let game = ItemGame::running_at(300.0, table).await;
    let mut updates = game.poll_updates(&openai.url);
    settle(&mut updates, &openai).await;
    prepare(&game);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    (settle(&mut updates, &openai).await, openai)
}

fn input_of(openai: &FakeOpenAi) -> String {
    openai.bodies()[0]["input"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_replacement_takes_the_next_items_place_with_its_reason() {
    let (update, _openai) = after_death(&answer(Some(STEELCAPS), Some(REASON)), |_| {}).await;

    assert_eq!(items(&update), [3047, 3102, 3157, 3003, 3089, 3135]);
    assert_eq!(update.build_change.as_deref(), Some(REASON));
    assert_eq!(update.game.unwrap().advice.unwrap().text, TIP);
}

// Rabadon's moves up in place of Banshee's, and Morello, the next completed item by
// share, keeps the Build its length.
#[tokio::test]
async fn a_replacement_from_later_in_the_build_moves_up_and_the_tail_refills() {
    let (update, _openai) = after_death(&answer(Some(RABADONS), Some(REASON)), |game| {
        game.buy(&[SORCERERS], 0.0);
    })
    .await;

    assert_eq!(items(&update), [3089, 3157, 3003, 3135, 3165]);
    assert_eq!(update.build_change.as_deref(), Some(REASON));
}

// No item the model names outside the Item candidates reaches the Build: see ADR 0004.
// Zhonya's is a candidate, but no boots, so it cannot take the boots' place.
#[tokio::test]
async fn an_item_outside_the_candidates_is_ignored() {
    for outside in [6692, 1058, ZHONYAS] {
        let (update, _openai) = after_death(&answer(Some(outside), Some(REASON)), |_| {}).await;

        assert_eq!(items(&update), FULL_BUILD, "{outside} changed the Build");
        assert_eq!(update.build_change, None);
        assert_eq!(update.game.unwrap().advice.unwrap().text, TIP);
    }
}

// A Blasting Wand bought in the visit that finished the boots starts Banshee's, which
// then stays, whatever the model says.
#[tokio::test]
async fn a_started_next_item_is_never_replaced() {
    let (update, openai) = after_death(&answer(Some(RABADONS), Some(REASON)), |game| {
        game.buy(&[SORCERERS], 0.0);
        game.buy(&[BLASTING_WAND], 0.0);
    })
    .await;

    assert_eq!(items(&update), [3102, 3157, 3003, 3089, 3135]);
    assert_eq!(update.build_change, None);
    let input = input_of(&openai);
    assert!(input.contains("Next item: Покров банши, started\n"));
    assert!(input.contains("Item candidates: none\n"));
}

// The Build Widget cannot show a change it has no reason for.
#[tokio::test]
async fn a_replacement_without_a_reason_is_ignored() {
    let (update, _openai) = after_death(&answer(Some(STEELCAPS), None), |_| {}).await;

    assert_eq!(items(&update), FULL_BUILD);
    assert_eq!(update.build_change, None);
}

#[tokio::test]
async fn the_death_prompt_offers_the_items_that_may_replace_the_next_one() {
    let (_update, openai) = after_death(&answer(None, None), |_| {}).await;

    let input = input_of(&openai);
    assert!(input.contains(
        "Build, in order: Сапоги чародея, Покров банши, item 3157, item 3003, item 3089, item 3135\n"
    ));
    assert!(input.contains("Next item: Сапоги чародея, not started\n"));
    assert!(input.contains(
        "Item candidate: 3047 Бронированные сапоги (Armor, Boots), bought in 30% of games, first at minute 9\n"
    ));
    assert_eq!(input.matches("Item candidate: ").count(), 1);
    assert!(input.contains("Death at 5:00: to Zed, magic damage 10%\n"));
    let instructions = openai.bodies()[0]["instructions"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(instructions.contains("`advice` never names an item"));
}

// Every death of the game goes in, so the model sees whose damage keeps killing.
#[tokio::test]
async fn the_death_prompt_lists_every_death_of_the_game() {
    let (_update, openai) = after_death(&answer(None, None), |game| {
        game.log_event(json!({
            "EventName": "ChampionKill",
            "EventTime": 125.0,
            "KillerName": "Foe4#EUW",
            "VictimName": ME,
            "Assisters": ["Foe2#EUW"],
        }));
    })
    .await;

    let input = input_of(&openai);
    assert!(input
        .contains("Death at 2:05: to Lux, magic damage 90%, assisted by Zed, magic damage 10%\n"));
    assert!(input.contains("Death at 5:00: to Zed, magic damage 10%\n"));
}

// The change shows a few seconds, then the Build Widget keeps the Build alone.
#[tokio::test]
async fn a_build_change_shows_for_ten_seconds() {
    let openai = FakeOpenAi::advising(&answer(Some(STEELCAPS), Some(REASON))).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_updates(&openai.url);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    game.set_time(309.0);
    assert_eq!(
        settle(&mut updates, &openai).await.build_change.as_deref(),
        Some(REASON)
    );
    game.set_time(310.0);
    let update = settle(&mut updates, &openai).await;
    assert_eq!(update.build_change, None);
    assert_eq!(items(&update)[0], STEELCAPS);
}

// ADR 0018 opens the Next item to the loading screen and a death alone: a Turning
// point's answer that names one shows its Advice and leaves the Build.
#[tokio::test]
async fn a_turning_point_answer_never_changes_the_build() {
    let openai = FakeOpenAi::advising(&answer(Some(STEELCAPS), Some(REASON))).await;
    let game = ItemGame::running_at(1200.0, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_updates(&openai.url);
    settle(&mut updates, &openai).await;

    game.log_event(json!({ "EventName": "BaronKill", "EventTime": 1200.0, "KillerName": ME }));
    let update = settle(&mut updates, &openai).await;

    assert!(!input_of(&openai).contains("Item candidate"));
    assert_eq!(items(&update), FULL_BUILD);
    assert_eq!(update.build_change, None);
    let shown = update.game.unwrap().advice.unwrap();
    assert_eq!(
        (shown.trigger, shown.text.as_str()),
        (Trigger::TurningPoint, TIP)
    );
}

// The overlay is hidden on the loading screen, so the change is seen once the game starts.
#[tokio::test]
async fn a_loading_screen_answer_changes_the_build_before_the_game() {
    let openai = FakeOpenAi::advising(&answer(Some(STEELCAPS), Some(REASON))).await;
    let game = ItemGame::at_loading_screen(&item_table(&magic_heavy())).await;
    let mut updates = game.poll_updates(&openai.url);

    let loading = settle(&mut updates, &openai).await;
    assert_eq!(items(&loading)[0], STEELCAPS);
    assert_eq!(loading.build_change, None);
    let input = input_of(&openai);
    assert!(input.contains("Next item: Сапоги чародея, not started\n"));
    assert!(input.contains("Enemy: Lux"));
    assert!(input.contains(", magic damage 90%\n"));

    game.start_game();
    let started = settle(&mut updates, &openai).await;
    assert_eq!(items(&started)[0], STEELCAPS);
    assert_eq!(started.build_change.as_deref(), Some(REASON));
    assert_eq!(
        started.game.unwrap().advice.unwrap().trigger,
        Trigger::LoadingScreen
    );
}

// Only a model's change has a reason to show: the player's own purchase has none.
#[tokio::test]
async fn an_off_build_purchase_shows_no_build_change() {
    let (update, _openai) = after_death(&answer(None, None), |game| {
        game.buy(&[SORCERERS], 0.0);
    })
    .await;

    assert_eq!(items(&update), [3102, 3157, 3003, 3089, 3135]);
    assert_eq!(update.build_change, None);
}

// Advice and its Build change come in one answer, and the voice reads both in one go,
// a colon spoken as a pause.
#[tokio::test]
async fn the_voice_reads_a_build_change_after_its_advice() {
    let (update, _openai) = after_death(&answer(Some(STEELCAPS), Some(REASON)), |_| {}).await;
    let mut reader = Reader::default();
    let advice = update.game.and_then(|game| game.advice);

    let speech = reader.follow(advice.as_ref(), update.build_change.as_deref());
    assert_eq!(
        speech,
        Some(Speech::Read(format!("{TIP} {REASON}").replace(':', ",")))
    );
    assert_eq!(
        reader.follow(advice.as_ref(), update.build_change.as_deref()),
        None
    );
    assert_eq!(reader.follow(None, None), Some(Speech::Stop));
}

// A table that cannot serve the game has no Build to change.
#[tokio::test]
async fn no_build_offers_no_item_candidates() {
    let openai = FakeOpenAi::advising(&answer(None, None)).await;
    let game = ItemGame::running_at(300.0, &Value::Null).await;
    let mut updates = game.poll_updates(&openai.url);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    let input = input_of(&openai);
    assert!(input.contains("Build: none\nItem candidates: none\n"));
}

// The team still heals a lot, so the model may not trade the anti-heal item away.
#[tokio::test]
async fn a_counter_item_leading_the_build_is_never_replaced() {
    let table = with_threats(item_table(&magic_heavy()), true, false);
    let bought = |game: &ItemGame| game.buy(&[SORCERERS, (3102, "Покров банши")], 0.0);
    let (update, _openai) =
        after_death_on(&table, &answer(Some(RABADONS), Some(REASON)), bought).await;

    assert_eq!(items(&update), [3165, ZHONYAS, 3003, RABADONS]);
    assert_eq!(update.build_change, None);
}
