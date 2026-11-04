//! Timing hints from the Objective schedule, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use std::path::Path;
use std::sync::Arc;

use lollipop_core::live::TimingHint;
use serde_json::{json, Value};

use support::fake_lcu::{FakeLcu, RANKED, SESSION, VERSION};
use support::game_clock::GameClock;
use support::hint_game::{
    banner_at, banner_over, banner_with_lockfile, baron_kill, dragon_kill, hints_over, no_client,
    HintPoll, CHAOS_PLAYER, ORDER_PLAYER,
};
use support::polling::start_playing;
use support::recorded_game::RecordedGame;
use support::server::serve;

const NO_TABLE: &str = "/nonexistent/benchmark-table.json";
// A kill time whose respawn falls clear of the fixed spawns, so only the pit can show.
const QUIET: f64 = 400.0;

fn session_in_queue(queue: u32) -> String {
    json!({ "gameData": { "queue": { "id": queue } } }).to_string()
}

fn version(version: &str) -> String {
    json!(version).to_string()
}

/// Whether a game at 4:45 shows the first drake's hint.
async fn shows_hints_with(lcu: &FakeLcu) -> bool {
    !banner_at(lcu, 285.0, Vec::new()).await.is_empty()
}

/// `count` elemental drakes, a minute apart, ending at `last_at`.
fn drakes(count: u32, last_at: f64, killer: &str) -> Vec<Value> {
    (0..count)
        .map(|back| dragon_kill((-60.0f64).mul_add(f64::from(back), last_at), "Air", killer))
        .rev()
        .collect()
}

// Protects the whole path on real data. Spawns are worked out by hand from the recording:
// drake kills at 7:14, 13:40, 19:21 and 27:09 and a Baron kill at 21:27. The drakes split
// 3 to 1, so no Elder comes.
#[tokio::test]
async fn replays_a_recorded_game() {
    let lcu = FakeLcu::start().await;
    let game = Arc::new(RecordedGame::load());
    let clock = GameClock::starting_at(game.first_game_time());
    let server_clock = clock.clone();
    let server_game = Arc::clone(&game);
    let url = serve(Box::new(move || server_game.payload_at(server_clock.now()))).await;
    let mut updates = start_playing(url, &lcu.lockfile(), Path::new(NO_TABLE), clock);

    let mut stretches: Vec<Stretch> = Vec::new();
    let mut in_game = false;
    loop {
        let (game_time, update) = updates.recv().await.unwrap();
        let Some(game) = update else {
            if in_game {
                break;
            }
            continue;
        };
        in_game = true;
        for hint in &game.hints {
            record(&mut stretches, game_time, hint);
        }
    }

    let spawns = [
        ("Drake", 300.0),
        ("Voidgrubs", 480.0),
        ("Drake", 734.08),
        ("Herald", 900.0),
        ("Drake", 1120.59),
        ("Baron", 1200.0),
        ("Drake", 1461.7),
        ("Baron", 1647.61),
        ("Drake", 1929.06),
    ];
    assert_eq!(stretches.len(), spawns.len());
    for (stretch, (objective, spawn)) in stretches.iter().zip(spawns) {
        assert_eq!(stretch.objective, objective, "spawn at {spawn}");
        assert_eq!(stretch.first_countdown, "0:45", "{objective} at {spawn}");
        assert_eq!(stretch.last_countdown, "0:01", "{objective} at {spawn}");
        // Polls are 1 s apart and a recorded state can lag the clock, so times are not exact.
        let appeared = stretch.first_seen - (spawn - 45.0);
        let disappeared = stretch.last_seen - spawn;
        assert!(appeared.abs() < 2.0, "{objective} at {spawn}: {appeared}");
        assert!(
            disappeared.abs() < 2.0,
            "{objective} at {spawn}: {disappeared}"
        );
    }
}

struct Stretch {
    objective: &'static str,
    first_countdown: String,
    last_countdown: String,
    first_seen: f64,
    last_seen: f64,
}

/// Joins a hint to the stretch its objective last showed in, unless time has moved on.
fn record(stretches: &mut Vec<Stretch>, game_time: f64, hint: &TimingHint) {
    let running = stretches
        .iter_mut()
        .rev()
        .find(|s| s.objective == hint.objective && game_time - s.last_seen < 5.0);
    if let Some(stretch) = running {
        stretch.last_countdown.clone_from(&hint.countdown);
        stretch.last_seen = game_time;
    } else {
        stretches.push(Stretch {
            objective: hint.objective,
            first_countdown: hint.countdown.clone(),
            last_countdown: hint.countdown.clone(),
            first_seen: game_time,
            last_seen: game_time,
        });
    }
}

// A hint at any other edge is a wrong hint: the lead is 45 s, and at the spawn it is gone.
#[tokio::test]
async fn a_hint_appears_45_s_before_its_spawn_counts_down_and_disappears_at_it() {
    let lcu = FakeLcu::start().await;
    for (objective, spawn) in [
        ("Drake", 300.0),
        ("Voidgrubs", 480.0),
        ("Herald", 900.0),
        ("Baron", 1200.0),
    ] {
        let line = |countdown: &str| vec![format!("{objective} {countdown}")];
        let at = |before: f64| banner_at(&lcu, spawn - before, Vec::new());
        assert_eq!(at(45.5).await, Vec::<String>::new(), "{objective}");
        assert_eq!(at(45.0).await, line("0:45"), "{objective}");
        assert_eq!(at(15.0).await, line("0:15"), "{objective}");
        assert_eq!(at(0.5).await, line("0:01"), "{objective}");
        assert_eq!(at(0.0).await, Vec::<String>::new(), "{objective}");
        assert_eq!(at(-1.0).await, Vec::<String>::new(), "{objective}");
    }
}

// Protects the countdown from one poll to the next, rather than a fresh start at each.
#[tokio::test]
async fn one_game_counts_down_poll_by_poll() {
    let lcu = FakeLcu::start().await;
    let polls = [1154.0, 1155.0, 1156.0, 1199.0, 1200.0]
        .into_iter()
        .map(|game_time| HintPoll {
            game_time,
            events: Vec::new(),
        })
        .collect();

    let banners = banner_over(&lcu.lockfile(), polls, |_| {}).await;

    let baron = |line: &str| vec![line.to_string()];
    assert_eq!(
        banners,
        [
            Vec::new(),
            baron("Baron 0:45"),
            baron("Baron 0:44"),
            baron("Baron 0:01"),
            Vec::new(),
        ]
    );
}

// With Fast Forward a kill can be logged late, so timing from the poll would drift.
#[tokio::test]
async fn a_respawn_runs_from_the_kills_event_time() {
    let lcu = FakeLcu::start().await;

    let drake = banner_at(&lcu, 700.0, vec![dragon_kill(420.0, "Air", ORDER_PLAYER)]).await;
    assert_eq!(drake, ["Drake 0:20"]);

    let baron = banner_at(&lcu, 1600.0, vec![baron_kill(1260.0)]).await;
    assert_eq!(baron, ["Baron 0:20"]);
}

// Only the last kill counts: a Baron killed since must not count down to its old respawn.
#[tokio::test]
async fn a_later_kill_replaces_the_earlier_spawn() {
    let lcu = FakeLcu::start().await;
    let events = vec![baron_kill(1260.0), baron_kill(1700.0)];

    assert_eq!(
        banner_at(&lcu, 1600.0, events.clone()).await,
        Vec::<String>::new()
    );
    assert_eq!(banner_at(&lcu, 2030.0, events).await, ["Baron 0:30"]);
}

#[tokio::test]
async fn a_teams_fourth_drake_moves_the_pit_to_elder_at_six_minutes() {
    let lcu = FakeLcu::start().await;
    let events = drakes(4, 1500.0, CHAOS_PLAYER);

    // Elder is up 6:00 after, at 1860. Drake's 5:00, at 1800, must show nothing.
    assert_eq!(
        banner_at(&lcu, 1790.0, events.clone()).await,
        Vec::<String>::new()
    );
    assert_eq!(
        banner_at(&lcu, 1815.0, events.clone()).await,
        ["Elder 0:45"]
    );
    assert_eq!(banner_at(&lcu, 1859.5, events).await, ["Elder 0:01"]);
}

// Four drakes between two teams are not one team's 4th: the Elder rule counts per team.
#[tokio::test]
async fn four_drakes_shared_between_teams_keep_the_pit_on_drakes() {
    let lcu = FakeLcu::start().await;
    let mut events = drakes(2, 900.0, ORDER_PLAYER);
    events.extend(drakes(2, 1500.0, CHAOS_PLAYER));

    assert_eq!(banner_at(&lcu, 1770.0, events).await, ["Drake 0:30"]);
}

#[tokio::test]
async fn an_elder_kill_brings_the_next_elder_six_minutes_later() {
    let lcu = FakeLcu::start().await;
    let mut events = drakes(4, 1500.0, ORDER_PLAYER);
    events.push(dragon_kill(1900.0, "Elder", CHAOS_PLAYER));

    assert_eq!(banner_at(&lcu, 2230.0, events).await, ["Elder 0:30"]);
}

// A kill by a minion has no player to join, and only its name gives the team.
#[tokio::test]
async fn minion_kills_count_for_the_team_their_name_gives() {
    let lcu = FakeLcu::start().await;
    let order_minion = "Minion_T100L0S39N0240";
    let chaos_minion = "Minion_T200L1S12N0100";

    let mut split = drakes(2, 900.0, order_minion);
    split.extend(drakes(2, 1500.0, chaos_minion));
    assert_eq!(banner_at(&lcu, 1770.0, split).await, ["Drake 0:30"]);

    let mut same_team = drakes(2, 900.0, order_minion);
    same_team.extend(drakes(2, 1500.0, ORDER_PLAYER));
    assert_eq!(banner_at(&lcu, 1830.0, same_team).await, ["Elder 0:30"]);
}

// The API's name for a player is not confirmed, so every name it could be must join.
#[tokio::test]
async fn a_killer_joins_to_the_player_list_by_any_of_their_names() {
    let lcu = FakeLcu::start().await;
    for killer in [CHAOS_PLAYER, "Named", "OldName"] {
        let events = drakes(4, 1500.0, killer);
        assert_eq!(
            banner_at(&lcu, 1830.0, events).await,
            ["Elder 0:30"],
            "{killer}"
        );
    }
}

#[tokio::test]
async fn overlapping_hints_stack_with_the_soonest_on_top() {
    let lcu = FakeLcu::start().await;
    let at_19_30 = 1170.0;

    // Baron is up at 20:00. One kill brings Drake back before it, the other after it.
    let drake_first = vec![dragon_kill(880.0, "Air", ORDER_PLAYER)];
    let baron_first = vec![dragon_kill(905.0, "Air", ORDER_PLAYER)];

    assert_eq!(
        banner_at(&lcu, at_19_30, drake_first).await,
        ["Drake 0:10", "Baron 0:30"]
    );
    assert_eq!(
        banner_at(&lcu, at_19_30, baron_first).await,
        ["Baron 0:30", "Drake 0:35"]
    );
}

// Without a team the Elder rule cannot be applied, so a later drake hint could be wrong.
#[tokio::test]
async fn a_dragon_kill_with_no_team_stops_drake_and_elder_hints() {
    let lcu = FakeLcu::start().await;
    let no_team = [
        json!({ "EventName": "DragonKill", "EventTime": QUIET, "DragonType": "Air" }),
        dragon_kill(QUIET, "Air", "SRU_RiftHerald"),
        dragon_kill(QUIET, "Air", "Teamless#EUW"),
        dragon_kill(QUIET, "Air", "Minion_T300L0S39N0240"),
        // Two players on opposite teams share this game name.
        dragon_kill(QUIET, "Air", "Twin"),
    ];
    for kill in no_team {
        let after_kill = banner_at(&lcu, QUIET + 280.0, vec![kill.clone()]).await;
        assert_eq!(after_kill, Vec::<String>::new(), "{kill}");
    }
}

#[tokio::test]
async fn a_kill_with_no_team_stops_the_pit_whatever_comes_after() {
    let lcu = FakeLcu::start().await;
    let mut events = drakes(3, 900.0, ORDER_PLAYER);
    events.push(dragon_kill(1000.0, "Air", "Unknown"));
    events.push(dragon_kill(1300.0, "Air", ORDER_PLAYER));
    assert_eq!(
        banner_at(&lcu, 1630.0, events.clone()).await,
        Vec::<String>::new()
    );
    assert_eq!(banner_at(&lcu, 1690.0, events).await, Vec::<String>::new());

    let unknown_elder = vec![dragon_kill(1800.0, "Elder", "Unknown")];
    assert_eq!(
        banner_at(&lcu, 2130.0, unknown_elder).await,
        Vec::<String>::new()
    );
}

// Only the pit stops: a Baron does not depend on who took a drake.
#[tokio::test]
async fn a_dragon_kill_with_no_team_leaves_other_objectives_on() {
    let lcu = FakeLcu::start().await;
    let events = vec![dragon_kill(QUIET, "Air", "Unknown")];

    assert_eq!(banner_at(&lcu, 1170.0, events).await, ["Baron 0:30"]);
}

// The app's first poll is already deep in the game, with no earlier poll to lean on.
#[tokio::test]
async fn a_start_mid_game_rebuilds_the_state_from_the_event_list() {
    let lcu = FakeLcu::start().await;

    let pit = drakes(4, 1500.0, ORDER_PLAYER);
    assert_eq!(banner_at(&lcu, 1830.0, pit).await, ["Elder 0:30"]);

    let stopped = vec![dragon_kill(QUIET, "Air", "Unknown")];
    assert_eq!(
        banner_at(&lcu, QUIET + 280.0, stopped).await,
        Vec::<String>::new()
    );

    assert_eq!(
        banner_at(&lcu, 1600.0, vec![baron_kill(1260.0)]).await,
        ["Baron 0:20"]
    );
}

#[tokio::test]
async fn hints_run_in_ranked_draft_blind_and_practice_tool() {
    let lcu = FakeLcu::start().await;
    for queue in [400, 420, 430, 440, 3140] {
        lcu.set(SESSION, &session_in_queue(queue));
        assert!(shows_hints_with(&lcu).await, "queue {queue}");
    }
}

#[tokio::test]
async fn any_other_queue_shows_no_hints() {
    let lcu = FakeLcu::start().await;
    // Swiftplay and ARAM, then a queue that never existed.
    for queue in [480, 450, 99999] {
        lcu.set(SESSION, &session_in_queue(queue));
        assert!(!shows_hints_with(&lcu).await, "queue {queue}");
    }
}

#[tokio::test]
async fn an_unknown_queue_shows_no_hints() {
    let lcu = FakeLcu::start().await;
    lcu.remove(SESSION);

    assert!(!shows_hints_with(&lcu).await);
}

#[tokio::test]
async fn an_unknown_patch_shows_no_hints() {
    let lcu = FakeLcu::start().await;
    lcu.remove(VERSION);
    assert!(!shows_hints_with(&lcu).await);

    lcu.set(VERSION, &version("not a version"));
    assert!(!shows_hints_with(&lcu).await);
}

#[tokio::test]
async fn hints_run_on_any_patch_within_a_season() {
    let lcu = FakeLcu::start().await;
    for patch in ["16.1.1+a", "16.21.8342754+a", "16.24.1+a", "16.99.1+a"] {
        lcu.set(VERSION, &version(patch));
        assert!(shows_hints_with(&lcu).await, "patch {patch}");
    }
}

// ADR 0005: the old schedule on a new Season once meant a hint for a Baron long up.
#[tokio::test]
async fn a_season_with_no_entry_shows_no_hints() {
    let lcu = FakeLcu::start().await;
    for patch in ["17.1.1+a", "26.21.1+a", "15.24.1+a"] {
        lcu.set(VERSION, &version(patch));
        assert!(!shows_hints_with(&lcu).await, "patch {patch}");
    }
}

#[tokio::test]
async fn an_unavailable_league_client_shows_no_hints() {
    let banner = banner_with_lockfile(&no_client(), 285.0, Vec::new()).await;

    assert_eq!(banner, Vec::<String>::new());
}

// Markers need the tier and hints do not, so losing it must cost only the Markers.
#[tokio::test]
async fn an_unreadable_tier_keeps_hints() {
    let lcu = FakeLcu::start().await;
    lcu.remove(RANKED);

    assert!(shows_hints_with(&lcu).await);
}

// The patch is read once: a hotfix landing mid-game must not change this game's schedule.
#[tokio::test]
async fn the_schedule_holds_for_the_whole_game() {
    let lcu = Arc::new(FakeLcu::start().await);
    let polls = [280.0, 285.0]
        .into_iter()
        .map(|game_time| HintPoll {
            game_time,
            events: Vec::new(),
        })
        .collect();
    let changing = Arc::clone(&lcu);

    let banners = banner_over(&lcu.lockfile(), polls, move |index| {
        if index == 1 {
            changing.set(VERSION, &version("17.1.1+a"));
        }
    })
    .await;

    assert_eq!(banners, [["Drake 0:20"], ["Drake 0:15"]]);
}

// A clock turned back is a new game, which must read its own schedule, not the last one's.
#[tokio::test]
async fn a_new_game_picks_its_own_schedule() {
    let lcu = Arc::new(FakeLcu::start().await);
    lcu.set(VERSION, &version("17.1.1+a"));
    let polls = [1000.0, 285.0]
        .into_iter()
        .map(|game_time| HintPoll {
            game_time,
            events: Vec::new(),
        })
        .collect();
    let changing = Arc::clone(&lcu);

    let banners = banner_over(&lcu.lockfile(), polls, move |index| {
        if index == 1 {
            changing.set(VERSION, &version("16.21.1+a"));
        }
    })
    .await;

    assert_eq!(
        banners,
        [Vec::<String>::new(), vec!["Drake 0:15".to_string()]]
    );
}

// The view reads this shape: see TimingHintState in ui/src/bridge.ts.
#[tokio::test]
async fn the_view_is_told_the_objective_and_its_countdown() {
    let lcu = FakeLcu::start().await;
    let polls = vec![HintPoll {
        game_time: 1185.0,
        events: Vec::new(),
    }];

    let hints = hints_over(&lcu.lockfile(), polls, |_| {}).await;

    assert_eq!(
        serde_json::to_value(&hints[0]).unwrap(),
        json!([{ "objective": "Baron", "countdown": "0:15", "elixir": null }])
    );
}
