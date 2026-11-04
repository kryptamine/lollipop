//! The Enemy tracker: headcount, the enemy jungler's whereabouts and Lane opponents'
//! trips to base, with League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::{
    EnemyTracker, Headcount, JunglerSighting, Place, ShownRecall, Whereabouts, WindowObjective,
};
use serde_json::json;

use support::fake_lcu::FakeLcu;
use support::hint_game::no_client;
use support::tracker_game::{
    champion_kill, objective_kill, trackers_over, TrackerPoll, ALLY_BOT, ENEMY_BOT, ENEMY_JUNGLER,
    ENEMY_MID, ENEMY_TOP, HEALTH_POTION, LONG_SWORD, MANAMUNE, ME, MURAMANA, SERRATED_DIRK,
};

async fn trackers(polls: Vec<TrackerPoll>) -> Vec<EnemyTracker> {
    trackers_over(&no_client(), polls).await
}

async fn tracker(poll: TrackerPoll) -> EnemyTracker {
    trackers(vec![poll]).await.pop().unwrap()
}

/// A tracker with a schedule: the fake client is in ranked solo on a patch that has one.
async fn scheduled_tracker(poll: TrackerPoll) -> EnemyTracker {
    let lcu = FakeLcu::start().await;
    trackers_over(&lcu.lockfile(), vec![poll])
        .await
        .pop()
        .unwrap()
}

fn seen(place: Place, ago: &str) -> JunglerSighting {
    JunglerSighting {
        champion: "Vi".to_string(),
        whereabouts: Whereabouts::Seen {
            place,
            ago: ago.to_string(),
        },
    }
}

fn zed_recall(ago: &str, bought: &[&str]) -> Vec<ShownRecall> {
    vec![ShownRecall {
        champion: "Zed".to_string(),
        ago: ago.to_string(),
        bought: bought.iter().map(ToString::to_string).collect(),
    }]
}

// --- Headcount ---

// The lead holds at least until the short side's first respawn, whatever the other does.
#[tokio::test]
async fn a_lead_lasts_until_the_short_side_is_back() {
    let poll = TrackerPoll::at(600.0)
        .dead(ENEMY_MID, 12.4)
        .dead(ENEMY_JUNGLER, 30.0)
        .dead(ALLY_BOT, 3.0);
    let headcount = tracker(poll).await.headcount.unwrap();

    assert_eq!((headcount.allies, headcount.enemies), (4, 3));
    assert_eq!(headcount.lasts, "0:13");
}

#[tokio::test]
async fn a_lead_against_the_player_shows_too() {
    let poll = TrackerPoll::at(600.0).dead(ME, 20.0).dead(ALLY_BOT, 8.0);
    let headcount = tracker(poll).await.headcount.unwrap();

    assert_eq!((headcount.allies, headcount.enemies), (3, 5));
    assert_eq!(headcount.lasts, "0:08");
}

#[tokio::test]
async fn even_sides_show_no_headcount() {
    let poll = TrackerPoll::at(600.0).dead(ME, 20.0).dead(ENEMY_TOP, 5.0);

    assert_eq!(tracker(poll).await.headcount, None);
}

// The Drake has been up since 5:00, as no one has killed it.
#[tokio::test]
async fn an_objective_up_during_the_lead_shows_beside_it() {
    let poll = TrackerPoll::at(600.0).dead(ENEMY_JUNGLER, 30.0);
    let headcount = scheduled_tracker(poll).await.headcount.unwrap();

    let drake_up = WindowObjective {
        objective: "Drake",
        spawns_in: None,
    };
    assert_eq!(headcount.objective, Some(drake_up));
}

// The Drake killed at 14:10 respawns at 19:10, before the lead may end at 19:30.
#[tokio::test]
async fn an_objective_spawning_during_the_lead_shows_its_countdown() {
    let poll = TrackerPoll::at(1100.0)
        .event(objective_kill("DragonKill", 850.0, "Garen#EUW"))
        .dead(ENEMY_JUNGLER, 70.0);
    let headcount = scheduled_tracker(poll).await.headcount.unwrap();

    let drake_soon = WindowObjective {
        objective: "Drake",
        spawns_in: Some("0:50".to_string()),
    };
    assert_eq!(headcount.objective, Some(drake_soon));
}

// A lead that ends before the Drake respawns is no window to take it.
#[tokio::test]
async fn an_objective_after_the_lead_does_not_show() {
    let poll = TrackerPoll::at(1100.0)
        .event(objective_kill("DragonKill", 850.0, "Garen#EUW"))
        .dead(ENEMY_JUNGLER, 30.0);
    let headcount = scheduled_tracker(poll).await.headcount.unwrap();

    assert_eq!(headcount.objective, None);
}

// --- Jungler ---

#[tokio::test]
async fn a_kill_the_jungler_helps_in_puts_them_in_that_lane() {
    let kill = champion_kill(400.0, ENEMY_MID, ALLY_BOT, &[ENEMY_JUNGLER]);
    let tracker = tracker(TrackerPoll::at(410.0).event(kill)).await;

    assert_eq!(tracker.jungler, Some(seen(Place::Bot, "0:10")));
}

#[tokio::test]
async fn an_objective_the_jungler_takes_puts_them_at_its_pit() {
    let drake = objective_kill("DragonKill", 400.0, ENEMY_JUNGLER);
    let tracker = tracker(TrackerPoll::at(405.0).event(drake)).await;

    assert_eq!(tracker.jungler, Some(seen(Place::Drake, "0:05")));
}

#[tokio::test]
async fn the_latest_sighting_wins() {
    let poll = TrackerPoll::at(500.0)
        .event(champion_kill(400.0, ENEMY_JUNGLER, ALLY_BOT, &[]))
        .event(objective_kill("HeraldKill", 470.0, ENEMY_JUNGLER));

    assert_eq!(
        tracker(poll).await.jungler,
        Some(seen(Place::Herald, "0:30"))
    );
}

#[tokio::test]
async fn a_dead_jungler_shows_their_respawn() {
    let tracker = tracker(TrackerPoll::at(600.0).dead(ENEMY_JUNGLER, 25.4)).await;

    let dead = JunglerSighting {
        champion: "Vi".to_string(),
        whereabouts: Whereabouts::Dead {
            back_in: "0:26".to_string(),
        },
    };
    assert_eq!(tracker.jungler, Some(dead));
}

#[tokio::test]
async fn a_respawned_jungler_is_in_base() {
    let trackers = trackers(vec![
        TrackerPoll::at(600.0).dead(ENEMY_JUNGLER, 1.0),
        TrackerPoll::at(601.0),
        TrackerPoll::at(611.0),
    ])
    .await;

    assert_eq!(trackers[1].jungler, Some(seen(Place::Base, "0:00")));
    assert_eq!(trackers[2].jungler, Some(seen(Place::Base, "0:10")));
}

#[tokio::test]
async fn a_jungler_seen_buying_is_in_base() {
    let trackers = trackers(vec![
        TrackerPoll::at(600.0).event(champion_kill(550.0, ENEMY_JUNGLER, ME, &[])),
        TrackerPoll::at(601.0)
            .event(champion_kill(550.0, ENEMY_JUNGLER, ME, &[]))
            .holding(ENEMY_JUNGLER, &[LONG_SWORD]),
    ])
    .await;

    assert_eq!(trackers[0].jungler, Some(seen(Place::Mid, "0:50")));
    assert_eq!(trackers[1].jungler, Some(seen(Place::Base, "0:00")));
}

// An old sighting would send the player looking where the jungler no longer is.
#[tokio::test]
async fn an_old_sighting_shows_nothing() {
    let kill = champion_kill(400.0, ENEMY_JUNGLER, ALLY_BOT, &[]);
    let trackers = trackers(vec![
        TrackerPoll::at(490.0).event(kill.clone()),
        TrackerPoll::at(491.0).event(kill),
    ])
    .await;

    assert_eq!(trackers[0].jungler, Some(seen(Place::Bot, "1:30")));
    assert_eq!(trackers[1].jungler, None);
}

#[tokio::test]
async fn a_jungler_not_yet_seen_shows_nothing() {
    let kill = champion_kill(400.0, ENEMY_MID, ALLY_BOT, &[]);

    assert_eq!(
        tracker(TrackerPoll::at(410.0).event(kill)).await.jungler,
        None
    );
}

// --- Recalls ---

#[tokio::test]
async fn a_lane_opponent_buying_is_back_in_base() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0).holding(ENEMY_MID, &[LONG_SWORD]),
        TrackerPoll::at(301.0).holding(ENEMY_MID, &[LONG_SWORD, SERRATED_DIRK]),
        TrackerPoll::at(340.0).holding(ENEMY_MID, &[LONG_SWORD, SERRATED_DIRK]),
        TrackerPoll::at(341.0).holding(ENEMY_MID, &[LONG_SWORD, SERRATED_DIRK]),
    ])
    .await;

    assert_eq!(trackers[0].recalls, []);
    assert_eq!(trackers[1].recalls, zed_recall("0:00", &["Serrated Dirk"]));
    assert_eq!(trackers[2].recalls, zed_recall("0:39", &["Serrated Dirk"]));
    // By then they are walking back into lane.
    assert_eq!(trackers[3].recalls, []);
}

#[tokio::test]
async fn purchases_on_one_trip_join() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0),
        TrackerPoll::at(301.0).holding(ENEMY_MID, &[SERRATED_DIRK]),
        TrackerPoll::at(305.0).holding(ENEMY_MID, &[SERRATED_DIRK, HEALTH_POTION]),
    ])
    .await;

    let both = zed_recall("0:04", &["Serrated Dirk", "Health Potion"]);
    assert_eq!(trackers[2].recalls, both);
}

#[tokio::test]
async fn starting_items_are_no_recall() {
    let trackers = trackers(vec![
        TrackerPoll::at(10.0),
        TrackerPoll::at(60.0).holding(ENEMY_MID, &[LONG_SWORD, HEALTH_POTION]),
    ])
    .await;

    assert_eq!(trackers[1].recalls, []);
}

// A respawn already puts them in base, and the Headcount shows it.
#[tokio::test]
async fn shopping_after_a_respawn_is_no_recall() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0).dead(ENEMY_MID, 1.0),
        TrackerPoll::at(301.0),
        TrackerPoll::at(320.0).holding(ENEMY_MID, &[SERRATED_DIRK]),
    ])
    .await;

    assert_eq!(trackers[2].recalls, []);
}

// Manamune turns into Muramana in lane, so the new item proves no trip to base.
#[tokio::test]
async fn an_item_upgrading_in_lane_is_no_recall() {
    let trackers = trackers(vec![
        TrackerPoll::at(900.0).holding(ENEMY_MID, &[MANAMUNE]),
        TrackerPoll::at(901.0).holding(ENEMY_MID, &[MURAMANA]),
    ])
    .await;

    assert_eq!(trackers[1].recalls, []);
}

#[tokio::test]
async fn a_recall_ends_with_a_death() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0),
        TrackerPoll::at(301.0).holding(ENEMY_MID, &[SERRATED_DIRK]),
        TrackerPoll::at(305.0)
            .holding(ENEMY_MID, &[SERRATED_DIRK])
            .dead(ENEMY_MID, 20.0),
    ])
    .await;

    assert_eq!(trackers[2].recalls, []);
}

#[tokio::test]
async fn a_recall_out_of_the_players_lane_is_not_shown() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0),
        TrackerPoll::at(301.0)
            .holding(ENEMY_TOP, &[SERRATED_DIRK])
            .holding(ENEMY_BOT, &[SERRATED_DIRK]),
    ])
    .await;

    assert_eq!(trackers[1].recalls, []);
}

// The view reads this shape: see EnemyTracker in ui/src/enemy-tracker.ts.
#[test]
fn the_view_is_told_the_tracker_in_this_shape() {
    let tracker = EnemyTracker {
        headcount: Some(Headcount {
            allies: 5,
            enemies: 4,
            lasts: "0:10".to_string(),
            objective: Some(WindowObjective {
                objective: "Baron",
                spawns_in: None,
            }),
        }),
        jungler: Some(JunglerSighting {
            champion: "Vi".to_string(),
            whereabouts: Whereabouts::Dead {
                back_in: "0:10".to_string(),
            },
        }),
        recalls: zed_recall("0:03", &["Serrated Dirk"]),
    };

    assert_eq!(
        serde_json::to_value(&tracker).unwrap(),
        json!({
            "headcount": {
                "allies": 5,
                "enemies": 4,
                "lasts": "0:10",
                "objective": { "objective": "Baron", "spawnsIn": null },
            },
            "jungler": { "champion": "Vi", "whereabouts": { "state": "dead", "backIn": "0:10" } },
            "recalls": [{ "champion": "Zed", "ago": "0:03", "bought": ["Serrated Dirk"] }],
        })
    );
    let seen = Whereabouts::Seen {
        place: Place::Drake,
        ago: "0:05".to_string(),
    };
    assert_eq!(
        serde_json::to_value(&seen).unwrap(),
        json!({ "state": "seen", "place": "drake", "ago": "0:05" })
    );
}
