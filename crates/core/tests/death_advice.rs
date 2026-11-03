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
use support::item_game::{
    item_table, magic_heavy, physical_heavy, settle, ItemGame, SETTLING_POLLS,
};
use support::loading_game::{Updates, ME};

const KILLER: &str = "Foe2#EUW";
const ASSISTER: &str = "Foe1#EUW";
const TIP: &str = "Zed ловит тебя у стены: собери Покров банши и держи E на его прыжок.";
const RESPAWN: f64 = 30.0;

fn answer(text: &str, item: Option<u32>) -> String {
    json!({ "advice": text, "item": item }).to_string()
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

// A death that teaches nothing is answered null, and a weak idea never fills the Widget.
#[tokio::test]
async fn null_advice_shows_nothing() {
    let null = json!({ "advice": null, "item": null }).to_string();

    assert_eq!(shown_after_death(&null).await, None);
}

// Every Trigger may stay silent, and generic tips are named so the model avoids them.
#[tokio::test]
async fn the_instructions_prefer_silence_to_a_generic_tip() {
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
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

// Plated Steelcaps is a candidate only against a physical-heavy team.
#[tokio::test]
async fn an_item_outside_the_candidates_drops_the_whole_advice() {
    assert_eq!(shown_after_death(&answer(TIP, Some(3047))).await, None);
}

// Code holds the Item to build: the model only explains it, so another candidate is out.
#[tokio::test]
async fn a_candidate_other_than_the_item_to_build_drops_the_whole_advice() {
    assert_eq!(shown_after_death(&answer(TIP, Some(3157))).await, None);
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
    assert!(input.contains(
        "Item to build: 3102 Покров банши, bought in 45% of games, first at minute 14\n"
    ));
    assert!(!input.contains("3157"));
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
async fn the_candidates_follow_the_enemy_damage_profile() {
    let input = input_of_death(&item_table(&physical_heavy()), |_| {}).await;

    assert!(input.contains("Item to build: 3047 Бронированные сапоги"));
    assert!(!input.contains("3102"));
}

// Owning Seraph's Embrace is owning the Archangel's Staff it grew from.
#[tokio::test]
async fn an_owned_item_is_never_the_item_to_build() {
    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(
            ME,
            &[
                (3102, "Покров банши"),
                (3157, "Песочные часы Жони"),
                (3040, "Объятия серафима"),
            ],
        );
    })
    .await;
    assert!(input.contains("Item to build: none\n"));

    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(ME, &[(3102, "Покров банши"), (3040, "Объятия серафима")]);
    })
    .await;
    assert!(input.contains("Item to build: 3157 Песочные часы Жони"));
}

// A held Rod outranks a candidate with an earlier minute.
#[tokio::test]
async fn a_held_component_makes_its_item_the_item_to_build() {
    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(ME, &[(1058, "Огромный жезл")]);
    })
    .await;

    assert!(input.contains("Item to build: 3157 Песочные часы Жони"));
}

// A component two candidates share ties neither, so the earlier minute still leads.
#[tokio::test]
async fn the_item_most_held_components_build_into_leads() {
    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(ME, &[(1052, "Усиливающий фолиант")]);
    })
    .await;
    assert!(input.contains("Item to build: 3102 Покров банши"));

    let input = input_of_death(&item_table(&magic_heavy()), |game| {
        game.give_items(
            ME,
            &[(1052, "Усиливающий фолиант"), (1058, "Огромный жезл")],
        );
    })
    .await;
    assert!(input.contains("Item to build: 3157 Песочные часы Жони"));
}

/// The inputs of two deaths to Zed, holding `before` at the first, whose Advice names
/// `shown`, and `after` at the second.
async fn inputs_of_two_deaths(
    shown: u32,
    before: &[(u32, &str)],
    after: &[(u32, &str)],
) -> [String; 2] {
    let openai = FakeOpenAi::advising(&answer(TIP, Some(shown))).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&magic_heavy()), &openai).await;
    game.give_items(ME, before);
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    game.respawn();
    game.give_items(ME, after);
    game.set_time(400.0);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    let bodies = openai.bodies();
    [0, 1].map(|index| bodies[index]["input"].as_str().unwrap().to_string())
}

// A death never swaps an item still being built for a new one, even on a tie.
#[tokio::test]
async fn an_item_shown_and_not_bought_wins_a_tie_in_components() {
    let [first, second] = inputs_of_two_deaths(
        3157,
        &[(1058, "Огромный жезл")],
        &[(1058, "Огромный жезл"), (1033, "Плащ отрицания")],
    )
    .await;

    assert!(first.contains("Item to build: 3157 Песочные часы Жони"));
    assert!(second.contains("Item to build: 3157 Песочные часы Жони"));
}

// The inventory beats the plan: a player who started another item is not told to
// switch back to the one shown.
#[tokio::test]
async fn components_of_another_item_outrank_the_item_shown() {
    let [first, second] = inputs_of_two_deaths(3102, &[], &[(1058, "Огромный жезл")]).await;

    assert!(first.contains("Item to build: 3102 Покров банши"));
    assert!(second.contains("Item to build: 3157 Песочные часы Жони"));
}

// A player who finished another item has left the plan: the item shown before would
// otherwise stay the Item to build for the rest of the game.
#[tokio::test]
async fn finishing_another_item_releases_the_item_shown() {
    let [first, second] = inputs_of_two_deaths(
        3157,
        &[(1058, "Огромный жезл")],
        &[(3089, "Смертельная шляпа Рабадона")],
    )
    .await;

    assert!(first.contains("Item to build: 3157 Песочные часы Жони"));
    assert!(second.contains("Item to build: 3102 Покров банши"));
}

// A player holds one pair of boots: boots shown and then passed over for another pair
// would otherwise stay the Item to build, and in the prompt, for the rest of the game.
#[tokio::test]
async fn boots_are_never_the_item_to_build_once_the_player_owns_a_pair() {
    let openai = FakeOpenAi::advising(&answer(TIP, Some(3047))).await;
    let (game, mut updates) = game_at_five_minutes(&item_table(&physical_heavy()), &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;
    game.respawn();
    game.give_items(ME, &[(3006, "Берсерки")]);
    game.set_time(400.0);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], RESPAWN);
    settle(&mut updates, &openai).await;

    let bodies = openai.bodies();
    assert!(bodies[0]["input"]
        .as_str()
        .unwrap()
        .contains("Item to build: 3047 Бронированные сапоги"));
    let input = bodies[1]["input"].as_str().unwrap();
    assert!(input.contains("Item to build: 3157 Песочные часы Жони"));
    assert!(!input.contains("Бронированные сапоги"));
}

// Without a table for the game, a death still gets Advice, but no item.
#[tokio::test]
async fn no_table_gives_no_item_to_build() {
    let input = input_of_death(&json!({}), |_| {}).await;

    assert!(input.contains("Item to build: none\n"));
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
    let openai = FakeOpenAi::advising(&answer(TIP, None)).await;
    let game = ItemGame::running_at(300.0, &item_table(&magic_heavy())).await;
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
            reply: Box::new(Reply::advising(&answer(LANE_PLAN, None))),
        },
        Reply::advising(&answer(TIP, None)),
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
