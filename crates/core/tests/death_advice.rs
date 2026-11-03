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

use support::death_game::{item_table, magic_heavy, physical_heavy, DeathGame};
use support::fake_openai::{FakeOpenAi, Reply};
use support::loading_game::{Updates, ME};

const KILLER: &str = "Foe2#EUW";
const ASSISTER: &str = "Foe1#EUW";
const TIP: &str = "Zed ловит тебя у стены: собери Покров банши и держи E на его прыжок.";
const RESPAWN: f64 = 30.0;
const SETTLING_POLLS: usize = 8;

fn answer(text: &str, item: Option<u32>) -> String {
    json!({ "advice": text, "item": item }).to_string()
}

/// Polls until OpenAI has answered every request and the poll loop has had time to take
/// the answers, and gives the Widget then.
async fn settle(updates: &mut Updates, openai: &FakeOpenAi) -> Option<ShownAdvice> {
    let mut last = None;
    for _ in 0..SETTLING_POLLS {
        last = updates.recv().await.unwrap();
    }
    while openai.answered() < openai.requests().len() {
        last = updates.recv().await.unwrap();
    }
    for _ in 0..3 {
        last = updates.recv().await.unwrap();
    }
    last.and_then(|game| game.advice)
}

/// A game at 5:00 on a magic-heavy enemy team, polled until it runs.
async fn game_at_five_minutes(table: &Value, openai: &FakeOpenAi) -> (DeathGame, Updates) {
    let game = DeathGame::running_at(300.0, table).await;
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
async fn input_of_death(table: &Value, prepare: impl FnOnce(&DeathGame)) -> String {
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
    let (game, mut updates) = game_at_five_minutes(table, &openai).await;
    prepare(&game);
    game.die_to(KILLER, &[ASSISTER], RESPAWN);
    settle(&mut updates, &openai).await;
    let bodies = openai.bodies();
    assert_eq!(bodies.len(), 1);
    bodies[0]["input"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn a_death_to_an_enemy_champion_shows_advice_with_the_item_icon() {
    let shown = shown_after_death(&answer(TIP, Some(3102))).await.unwrap();

    assert_eq!(shown.trigger, Trigger::Death);
    assert_eq!(shown.text, TIP);
    assert_eq!(
        shown.icon.as_deref(),
        Some("https://ddragon.leagueoflegends.com/cdn/16.21.1/img/item/3102.png")
    );
}

#[tokio::test]
async fn advice_without_an_item_shows_no_icon() {
    let shown = shown_after_death(&answer(TIP, None)).await.unwrap();

    assert_eq!(shown.text, TIP);
    assert_eq!(shown.icon, None);
}

// Plated Steelcaps is a candidate only against a physical-heavy team.
#[tokio::test]
async fn an_item_outside_the_candidates_drops_the_whole_advice() {
    assert_eq!(shown_after_death(&answer(TIP, Some(3047))).await, None);
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
    assert!(input.contains("Killer: Zed, MIDDLE, level 1, KDA 1/2/3, items Затмение\n"));
    assert!(input.contains("Assister: LeeSin, JUNGLE"));
    assert!(input.contains("Deaths to this killer this game: 1\n"));
    assert_eq!(input.matches("Enemy: ").count(), 5);
    assert!(input.contains("- 3102 Покров банши: bought in 45% of games, first at minute 18\n"));
    for name in ["#EUW", "Foe", "Friend", "Me,"] {
        assert!(!input.contains(name), "{name} is in the prompt");
    }
}

#[tokio::test]
async fn the_candidates_follow_the_enemy_damage_profile() {
    let input = input_of_death(&item_table(&physical_heavy()), |_| {}).await;

    assert!(input.contains("- 3047 Бронированные сапоги"));
    assert!(!input.contains("3102"));
}

// Owning Seraph's Embrace is owning the Archangel's Staff it grew from.
#[tokio::test]
async fn owned_items_are_no_candidates() {
    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(
            ME,
            &[(3157, "Песочные часы Жони"), (3040, "Объятия серафима")],
        );
    })
    .await;

    assert!(input.contains("- 3102 Покров банши"));
    assert!(!input.contains("- 3157"));
    assert!(!input.contains("- 3003"));
}

// Without a table for the game, a death still gets Advice, but no item.
#[tokio::test]
async fn no_table_lists_no_candidates() {
    let input = input_of_death(&json!({}), |_| {}).await;

    assert!(input.contains("Item candidates: none\n"));
}

#[tokio::test]
async fn deaths_to_the_same_killer_are_counted() {
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
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

// A turret, a minion or a monster gets no Advice, and neither does an ally's death.
#[tokio::test]
async fn other_kills_ask_nothing() {
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
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
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
    let game = DeathGame::running_at(300.0, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_with(&openai.url, |trigger| trigger != Trigger::Death);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

// An app started while the player is dead does not advise on that death.
#[tokio::test]
async fn a_death_before_the_first_poll_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
    let game = DeathGame::running_at(300.0, &item_table(&magic_heavy())).await;
    game.die_to(KILLER, &[], RESPAWN);
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn an_answer_after_respawn_is_dropped() {
    let openai = FakeOpenAi::advising_when_released(&answer(TIP, None)).await;
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
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
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
        Reply::advising(&answer("Первая смерть.", None)),
        Reply::advising(&answer("Вторая смерть.", None)),
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

#[tokio::test]
async fn an_item_shown_earlier_goes_in_the_next_prompt() {
    let openai = FakeOpenAi::advising(&answer(TIP, Some(3102))).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    game.respawn();
    game.set_time(400.0);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    game.respawn();
    game.give_items(ME, &[(3102, "Покров банши")]);
    game.set_time(500.0);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    let inputs: Vec<String> = openai
        .bodies()
        .iter()
        .map(|body| body["input"].as_str().unwrap().to_string())
        .collect();
    assert!(!inputs[0].contains("Earlier recommended"));
    assert!(inputs[1].contains("Earlier recommended: Покров банши, not bought\n"));
    assert!(inputs[2].contains("Earlier recommended: Покров банши, bought\n"));
}

const LANE_PLAN: &str = "Играй от Electrocute.";

#[tokio::test]
async fn death_advice_replaces_the_loading_screen_advice() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(LANE_PLAN, None)),
        Reply::advising(&answer(TIP, None)),
    ])
    .await;
    let game = DeathGame::at_loading_screen(&item_table(&magic_heavy())).await;
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
            reply: Box::new(Reply::advising(&answer(LANE_PLAN, None))),
        },
        Reply::advising(&answer(TIP, None)),
    ])
    .await;
    let game = DeathGame::at_loading_screen(&item_table(&magic_heavy())).await;
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
