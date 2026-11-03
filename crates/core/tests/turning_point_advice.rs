//! Advice on a Turning point, with League and OpenAI faked over HTTP.
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

const ALLY: &str = "Friend1#EUW";
const KILLER: &str = "Foe2#EUW";
const PLAN: &str = "Сдвигайся к барону: у врага нет двоих.";
const NEXT_PLAN: &str = "Толкай нижнюю линию, пока барон у вас.";
// Into the game, past the laning phase so no 14:00 moment interferes.
const LATE: f64 = 1200.0;

fn answer(text: Option<&str>) -> String {
    json!({ "advice": text, "item": null }).to_string()
}

fn baron(at: f64, killer: &str) -> Value {
    json!({ "EventName": "BaronKill", "EventTime": at, "KillerName": killer })
}

fn turret(at: f64, turret: &str, killer: &str) -> Value {
    json!({ "EventName": "TurretKilled", "EventTime": at, "TurretKilled": turret, "KillerName": killer })
}

/// A game at `game_time`, polled until it runs, with only the Turning point switch on.
async fn game_at(game_time: f64, openai: &FakeOpenAi) -> (ItemGame, Updates) {
    let game = ItemGame::running_at(game_time, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_with(&openai.url, |trigger| trigger == Trigger::TurningPoint);
    settle(&mut updates, openai).await;
    (game, updates)
}

/// `event` logged at the game's current time, and the Widget once it settles.
async fn moment(
    game: &ItemGame,
    updates: &mut Updates,
    openai: &FakeOpenAi,
    event: Value,
) -> Option<ShownAdvice> {
    game.log_event(event);
    settle(updates, openai).await
}

fn inputs(openai: &FakeOpenAi) -> Vec<String> {
    openai
        .bodies()
        .iter()
        .map(|body: &Value| body["input"].as_str().unwrap().to_string())
        .collect()
}

#[tokio::test]
async fn a_baron_shows_turning_point_advice_without_an_icon() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;

    let shown = moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;

    let shown = shown.unwrap();
    assert_eq!(shown.trigger, Trigger::TurningPoint);
    assert_eq!(shown.text, PLAN);
    assert_eq!(shown.icon, None);
}

#[tokio::test]
async fn the_prompt_carries_the_moment_the_score_and_both_teams_without_names() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let game = ItemGame::running_at(LATE, &item_table(&magic_heavy())).await;
    game.log_event(json!({ "EventName": "DragonKill", "EventTime": 400.0, "DragonType": "Fire", "KillerName": KILLER }));
    game.log_event(turret(
        600.0,
        "Turret_TChaos_L1_P3_2254202041_0",
        "Minion_T100L1S20N0100",
    ));
    game.log_event(json!({ "EventName": "HeraldKill", "EventTime": 800.0, "KillerName": ALLY }));
    let mut updates = game.poll_with(&openai.url, |trigger| trigger == Trigger::TurningPoint);
    settle(&mut updates, &openai).await;

    moment(
        &game,
        &mut updates,
        &openai,
        json!({ "EventName": "DragonKill", "EventTime": LATE, "DragonType": "Earth", "KillerName": ALLY }),
    )
    .await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 1);
    let input = &inputs[0];
    assert!(input.contains("Language: Russian\n"));
    assert!(input.contains("Moment: a drake was killed: Earth\nTaken by: the player's team\n"));
    assert!(input.contains("Game time: 20:00\n"));
    assert!(input.contains("Player: Ahri, MIDDLE, level 1, KDA 1/2/3, no items, gold 1250\n"));
    assert_eq!(input.matches("Ally: ").count(), 4);
    assert_eq!(input.matches("Enemy: ").count(), 5);
    assert!(input.contains("Ally: Vi, JUNGLE"));
    assert!(input.contains("Enemy: Zed, MIDDLE, level 1, KDA 5/1/0, no items\n"));
    assert!(input.contains(
        "Player's team has taken: dragons Earth; Herald 1; Baron 0; turrets 1; inhibitors 0\n"
    ));
    assert!(input.contains(
        "Enemy team has taken: dragons Fire; Herald 0; Baron 0; turrets 0; inhibitors 0\n"
    ));
    assert!(!input.contains("Item candidates"));
    for name in ["#EUW", "Foe", "Friend", "Me,"] {
        assert!(!input.contains(name), "{name} is in the prompt");
    }
}

// The model draws the difference between taking an objective and losing it.
#[tokio::test]
async fn the_prompt_says_the_enemy_team_took_an_inhibitor() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;

    moment(
        &game,
        &mut updates,
        &openai,
        json!({ "EventName": "InhibKilled", "EventTime": LATE, "InhibKilled": "Inhib_TOrder_L1_P1_1931666598_0", "KillerName": "Minion_T200L1S40N0300" }),
    )
    .await;

    let input = &inputs(&openai)[0];
    assert!(input.contains("Moment: an inhibitor was destroyed\nTaken by: the enemy team\n"));
    assert!(input.contains(
        "Enemy team has taken: dragons none; Herald 0; Baron 0; turrets 0; inhibitors 1\n"
    ));
}

#[tokio::test]
async fn an_ace_says_which_team_aced() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;

    moment(
        &game,
        &mut updates,
        &openai,
        json!({ "EventName": "Ace", "EventTime": LATE, "Acer": KILLER, "AcingTeam": "CHAOS" }),
    )
    .await;

    let input = &inputs(&openai)[0];
    assert!(input.contains("Moment: an ace: a whole team is dead\nTaken by: the enemy team\n"));
}

// Earlier Advice texts let the model avoid saying the same thing twice.
#[tokio::test]
async fn the_prompt_carries_earlier_advice_texts() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;
    game.set_time(LATE + 200.0);

    moment(
        &game,
        &mut updates,
        &openai,
        json!({ "EventName": "HeraldKill", "EventTime": LATE + 200.0, "KillerName": KILLER }),
    )
    .await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 2);
    assert!(!inputs[0].contains("Earlier Advice"));
    assert!(inputs[1].contains(&format!("Earlier Advice: {PLAN}\n")));
}

// Starting the app mid-game must not ask about objectives taken long ago.
#[tokio::test]
async fn events_before_the_first_poll_ask_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let game = ItemGame::running_at(LATE, &item_table(&magic_heavy())).await;
    game.log_event(baron(LATE - 5.0, ALLY));
    let mut updates = game.poll_with(&openai.url, |trigger| trigger == Trigger::TurningPoint);
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn the_first_turret_ends_the_laning_phase_once() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(600.0, &openai).await;
    moment(
        &game,
        &mut updates,
        &openai,
        turret(600.0, "Turret_TChaos_L1_P3_2254202041_0", ALLY),
    )
    .await;
    assert_eq!(openai.requests().len(), 1);
    assert!(inputs(&openai)[0].contains(
        "Moment: the laning phase ends: the first turret of the game fell\nTaken by: the player's team\n"
    ));

    game.set_time(900.0);
    moment(
        &game,
        &mut updates,
        &openai,
        turret(900.0, "Turret_TOrder_L1_P3_1242677625_0", KILLER),
    )
    .await;
    assert_eq!(openai.requests().len(), 1);
}

#[tokio::test]
async fn fourteen_minutes_ends_the_laning_phase_with_every_turret_up() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(830.0, &openai).await;
    game.set_time(840.0);
    settle(&mut updates, &openai).await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 1);
    assert!(inputs[0].contains("Moment: the laning phase ends at 14:00\n"));
    assert!(!inputs[0].contains("Taken by"));
}

#[tokio::test]
async fn levels_6_11_and_16_fire_and_others_do_not() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.set_level(5);
    settle(&mut updates, &openai).await;
    assert!(openai.requests().is_empty());

    game.set_level(6);
    settle(&mut updates, &openai).await;
    game.set_time(LATE + 200.0);
    game.set_level(10);
    settle(&mut updates, &openai).await;
    game.set_level(11);
    settle(&mut updates, &openai).await;
    game.set_time(LATE + 400.0);
    game.set_level(16);
    settle(&mut updates, &openai).await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 3);
    assert!(inputs[0].contains("Moment: the player reached level 6\n"));
    assert!(inputs[1].contains("Moment: the player reached level 11\n"));
    assert!(inputs[2].contains("Moment: the player reached level 16\n"));
    assert!(!inputs[0].contains("Taken by"));
}

#[tokio::test]
async fn void_grubs_kills_and_later_turrets_ask_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(json!({ "EventName": "HordeKill", "EventTime": LATE, "KillerName": ALLY }));
    game.log_event(json!({ "EventName": "ChampionKill", "EventTime": LATE, "KillerName": ALLY, "VictimName": KILLER }));
    game.log_event(json!({ "EventName": "FirstBlood", "EventTime": LATE, "Recipient": ALLY }));
    game.log_event(
        json!({ "EventName": "Multikill", "EventTime": LATE, "KillerName": ALLY, "KillStreak": 2 }),
    );
    game.log_event(turret(LATE, "Turret_TChaos_L1_P2_2237424422_0", ALLY));
    game.log_event(json!({ "EventName": "InhibRespawned", "EventTime": LATE, "InhibRespawned": "Barracks_T2_L1" }));
    settle(&mut updates, &openai).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn a_moment_within_3_minutes_of_one_shown_is_skipped() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;

    game.set_time(LATE + 100.0);
    moment(&game, &mut updates, &openai, baron(LATE + 100.0, KILLER)).await;
    assert_eq!(openai.requests().len(), 1);

    game.set_time(LATE + 180.0);
    moment(&game, &mut updates, &openai, baron(LATE + 180.0, KILLER)).await;
    assert_eq!(openai.requests().len(), 2);
}

// A Baron right after an Ace is not lost to the spacing.
#[tokio::test]
async fn a_moment_within_30_seconds_replaces_the_one_shown() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(Some(PLAN))),
        Reply::advising(&answer(Some(NEXT_PLAN))),
    ])
    .await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    let ace = json!({ "EventName": "Ace", "EventTime": LATE, "Acer": ALLY, "AcingTeam": "ORDER" });
    let shown = moment(&game, &mut updates, &openai, ace).await.unwrap();
    assert_eq!(shown.text, PLAN);

    game.set_time(LATE + 30.0);
    let shown = moment(&game, &mut updates, &openai, baron(LATE + 30.0, ALLY)).await;

    assert_eq!(openai.requests().len(), 2);
    assert_eq!(shown.unwrap().text, NEXT_PLAN);
}

#[tokio::test]
async fn null_advice_shows_nothing_and_starts_no_window() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(None)),
        Reply::advising(&answer(Some(PLAN))),
    ])
    .await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    let shown = moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;
    assert_eq!(shown, None);

    game.set_time(LATE + 60.0);
    let shown = moment(&game, &mut updates, &openai, baron(LATE + 60.0, KILLER)).await;

    assert_eq!(openai.requests().len(), 2);
    assert_eq!(shown.unwrap().text, PLAN);
}

#[tokio::test]
async fn an_answer_over_15_seconds_after_the_moment_is_dropped_and_starts_no_window() {
    let openai = FakeOpenAi::advising_when_released(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(baron(LATE, ALLY));
    while openai.requests().is_empty() {
        updates.recv().await.unwrap();
    }
    game.set_time(LATE + 16.0);
    openai.release();
    assert_eq!(settle(&mut updates, &openai).await, None);

    game.set_time(LATE + 60.0);
    moment(&game, &mut updates, &openai, baron(LATE + 60.0, KILLER)).await;
    assert_eq!(openai.requests().len(), 2);
}

#[tokio::test]
async fn an_answer_after_a_death_is_dropped_when_alive_at_the_moment() {
    let openai = FakeOpenAi::advising_when_released(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(baron(LATE, ALLY));
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
async fn a_moment_while_dead_shows_advice() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.die_to(KILLER, &[], 40.0);
    settle(&mut updates, &openai).await;

    let shown = moment(&game, &mut updates, &openai, baron(LATE, KILLER)).await;

    assert_eq!(shown.unwrap().trigger, Trigger::TurningPoint);
}

#[tokio::test]
async fn turning_point_advice_does_not_replace_death_advice() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(Some("Не стой у стены против Zed."))),
        Reply::advising(&answer(Some(PLAN))),
    ])
    .await;
    let game = ItemGame::running_at(LATE, &item_table(&magic_heavy())).await;
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], 40.0);
    settle(&mut updates, &openai).await;

    let shown = moment(&game, &mut updates, &openai, baron(LATE, KILLER)).await;

    assert_eq!(openai.answered(), 2);
    assert_eq!(shown.unwrap().trigger, Trigger::Death);
}

// Rank keeps it off the Widget, so the 3 min window does not start either.
#[tokio::test]
async fn a_turning_point_kept_off_by_rank_starts_no_window() {
    let openai = FakeOpenAi::replying(vec![
        Reply::advising(&answer(Some("Не стой у стены против Zed."))),
        Reply::advising(&answer(Some(PLAN))),
        Reply::advising(&answer(Some(NEXT_PLAN))),
    ])
    .await;
    let game = ItemGame::running_at(LATE, &item_table(&magic_heavy())).await;
    let mut updates = game.poll(&openai.url);
    settle(&mut updates, &openai).await;
    game.die_to(KILLER, &[], 40.0);
    settle(&mut updates, &openai).await;
    moment(&game, &mut updates, &openai, baron(LATE, KILLER)).await;
    game.respawn();
    game.set_time(LATE + 60.0);

    let ace = json!({ "EventName": "Ace", "EventTime": LATE + 60.0, "Acer": KILLER, "AcingTeam": "CHAOS" });
    let shown = moment(&game, &mut updates, &openai, ace).await;

    assert_eq!(openai.requests().len(), 3);
    assert_eq!(shown.unwrap().text, NEXT_PLAN);
}

#[tokio::test]
async fn the_turning_point_switch_off_asks_nothing() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let game = ItemGame::running_at(LATE, &item_table(&magic_heavy())).await;
    let mut updates = game.poll_with(&openai.url, |trigger| trigger != Trigger::TurningPoint);
    settle(&mut updates, &openai).await;

    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;

    assert!(openai.requests().is_empty());
}

// Live Client Data has no map positions, so a lane or an ally to join is a guess.
#[tokio::test]
async fn the_instructions_say_the_players_position_is_unknown() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;

    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;

    let instructions = openai.bodies()[0]["instructions"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(instructions.contains("Where the player is on the map is unknown"));
}

// A player in a fight cannot act on a macro plan, and a Baron taken then is not lost.
#[tokio::test]
async fn a_moment_within_20_seconds_of_the_players_kill_waits_for_the_fight_to_pass() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(json!({ "EventName": "ChampionKill", "EventTime": LATE, "KillerName": ME, "VictimName": KILLER, "Assisters": [] }));
    game.set_time(LATE + 5.0);

    let shown = moment(&game, &mut updates, &openai, baron(LATE + 5.0, ALLY)).await;
    assert_eq!(shown, None);
    game.set_time(LATE + 19.0);
    settle(&mut updates, &openai).await;
    assert!(openai.requests().is_empty());

    game.set_time(LATE + 20.0);
    let shown = settle(&mut updates, &openai).await;

    assert_eq!(openai.requests().len(), 1);
    assert_eq!(shown.unwrap().text, PLAN);
}

#[tokio::test]
async fn a_moment_right_after_the_players_assist_waits_too() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(json!({ "EventName": "ChampionKill", "EventTime": LATE, "KillerName": ALLY, "VictimName": KILLER, "Assisters": [ME] }));

    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;

    assert!(openai.requests().is_empty());
}

#[tokio::test]
async fn a_newer_moment_in_the_fight_replaces_the_one_held() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(json!({ "EventName": "ChampionKill", "EventTime": LATE, "KillerName": ME, "VictimName": KILLER, "Assisters": [] }));
    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;
    game.set_time(LATE + 5.0);
    moment(
        &game,
        &mut updates,
        &openai,
        json!({ "EventName": "InhibKilled", "EventTime": LATE + 5.0, "InhibKilled": "Barracks_T2_L1", "KillerName": ALLY }),
    )
    .await;

    game.set_time(LATE + 20.0);
    settle(&mut updates, &openai).await;

    let inputs = inputs(&openai);
    assert_eq!(inputs.len(), 1);
    assert!(inputs[0].contains("Moment: an inhibitor was destroyed\n"));
}

// A dead player is out of the fight, so the plan held for it is asked at once.
#[tokio::test]
async fn a_death_after_the_players_kill_asks_the_moment_held() {
    let openai = FakeOpenAi::advising(&answer(Some(PLAN))).await;
    let (game, mut updates) = game_at(LATE, &openai).await;
    game.log_event(json!({ "EventName": "ChampionKill", "EventTime": LATE, "KillerName": ME, "VictimName": KILLER, "Assisters": [] }));
    moment(&game, &mut updates, &openai, baron(LATE, ALLY)).await;
    assert!(openai.requests().is_empty());

    game.die_to(KILLER, &[], 40.0);
    let shown = settle(&mut updates, &openai).await;

    assert_eq!(openai.requests().len(), 1);
    assert_eq!(shown.unwrap().trigger, Trigger::TurningPoint);
}
