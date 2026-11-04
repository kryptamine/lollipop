//! The Enemy tracker: Headcount, Jungler sighting and the Lane opponent's Recall, with
//! League faked over HTTP.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    unreachable_pub,
    reason = "a failed test should panic, and a test crate has no public API"
)]

#[allow(dead_code, reason = "each test crate uses only part of the harness")]
mod support;

use lollipop_core::live::{
    EnemyTracker, Headcount, JunglerSighting, Place, SafeLane, ShownRecall, Whereabouts,
    WindowObjective,
};
use serde_json::json;

use support::fake_lcu::FakeLcu;
use support::hint_game::no_client;
use support::tracker_game::{
    champion_kill, objective_kill, trackers_over, TrackerPoll, ALLY_BOT, BISCUIT, ENEMY_BOT,
    ENEMY_JUNGLER, ENEMY_MID, ENEMY_TOP, HEALTH_POTION, LONG_SWORD, MANAMUNE, ME, MURAMANA,
    SERRATED_DIRK,
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

fn whereabouts(tracker: &EnemyTracker) -> Option<Whereabouts> {
    tracker.jungler.as_ref()?.whereabouts.clone()
}

fn seen(place: Place, ago: &str) -> Whereabouts {
    Whereabouts::Seen {
        place,
        ago: ago.to_string(),
    }
}

fn zed_recall(back_in: &str, bought: &[&str]) -> Vec<ShownRecall> {
    vec![ShownRecall {
        champion: "Zed".to_string(),
        back_in: back_in.to_string(),
        teleport: false,
        bought: bought.iter().map(ToString::to_string).collect(),
    }]
}

#[tokio::test]
async fn one_down_leads_until_that_one_is_back() {
    let poll = TrackerPoll::at(600.0)
        .dead(ENEMY_MID, 12.4)
        .dead(ENEMY_JUNGLER, 30.0)
        .dead(ALLY_BOT, 3.0);
    let headcount = tracker(poll).await.headcount.unwrap();

    assert_eq!((headcount.allies, headcount.enemies), (4, 3));
    assert_eq!(headcount.lead_ends_in, "0:13");
}

// At 5v3 the first respawn still leaves a 5v4: the lead ends with the second.
#[tokio::test]
async fn two_down_lead_until_both_are_back() {
    let poll = TrackerPoll::at(600.0)
        .dead(ENEMY_MID, 12.4)
        .dead(ENEMY_JUNGLER, 30.0);
    let headcount = tracker(poll).await.headcount.unwrap();

    assert_eq!((headcount.allies, headcount.enemies), (5, 3));
    assert_eq!(headcount.lead_ends_in, "0:30");
}

#[tokio::test]
async fn a_lead_against_the_player_shows_too() {
    let poll = TrackerPoll::at(600.0).dead(ME, 20.0).dead(ALLY_BOT, 8.0);
    let headcount = tracker(poll).await.headcount.unwrap();

    assert_eq!((headcount.allies, headcount.enemies), (3, 5));
    assert_eq!(headcount.lead_ends_in, "0:20");
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
        objective: Place::Drake,
        spawns_in: None,
    };
    assert_eq!(headcount.objective, Some(drake_up));
}

// The Drake killed at 14:10 respawns at 19:10, before the lead ends at 19:30.
#[tokio::test]
async fn an_objective_spawning_during_the_lead_shows_its_countdown() {
    let poll = TrackerPoll::at(1100.0)
        .event(objective_kill("DragonKill", 850.0, "Garen#EUW"))
        .dead(ENEMY_JUNGLER, 70.0);
    let headcount = scheduled_tracker(poll).await.headcount.unwrap();

    let drake_soon = WindowObjective {
        objective: Place::Drake,
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

#[tokio::test]
async fn a_kill_the_jungler_helps_in_puts_them_in_that_lane() {
    let kill = champion_kill(400.0, ENEMY_MID, ALLY_BOT, &[ENEMY_JUNGLER]);
    let tracker = tracker(TrackerPoll::at(410.0).event(kill)).await;

    assert_eq!(whereabouts(&tracker), Some(seen(Place::Bot, "0:10")));
}

#[tokio::test]
async fn an_objective_the_jungler_takes_puts_them_at_its_pit() {
    let drake = objective_kill("DragonKill", 400.0, ENEMY_JUNGLER);
    let tracker = tracker(TrackerPoll::at(405.0).event(drake)).await;

    assert_eq!(whereabouts(&tracker), Some(seen(Place::Drake, "0:05")));
}

#[tokio::test]
async fn voidgrubs_put_the_jungler_top_side() {
    let grubs = objective_kill("HordeKill", 400.0, ENEMY_JUNGLER);
    let tracker = tracker(TrackerPoll::at(405.0).event(grubs)).await;

    assert_eq!(whereabouts(&tracker), Some(seen(Place::Voidgrubs, "0:05")));
}

#[tokio::test]
async fn the_latest_sighting_wins() {
    let poll = TrackerPoll::at(500.0)
        .event(champion_kill(400.0, ENEMY_JUNGLER, ALLY_BOT, &[]))
        .event(objective_kill("HeraldKill", 470.0, ENEMY_JUNGLER));

    assert_eq!(
        whereabouts(&tracker(poll).await),
        Some(seen(Place::Herald, "0:30"))
    );
}

#[tokio::test]
async fn a_dead_jungler_shows_their_respawn() {
    let tracker = tracker(TrackerPoll::at(600.0).dead(ENEMY_JUNGLER, 25.4)).await;

    let dead = Whereabouts::Dead {
        back_in: "0:26".to_string(),
    };
    assert_eq!(whereabouts(&tracker), Some(dead));
}

#[tokio::test]
async fn a_respawned_jungler_is_in_base() {
    let trackers = trackers(vec![
        TrackerPoll::at(600.0).dead(ENEMY_JUNGLER, 1.0),
        TrackerPoll::at(601.0),
        TrackerPoll::at(611.0),
    ])
    .await;

    assert_eq!(whereabouts(&trackers[1]), Some(seen(Place::Base, "0:00")));
    assert_eq!(whereabouts(&trackers[2]), Some(seen(Place::Base, "0:10")));
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

    assert_eq!(whereabouts(&trackers[0]), Some(seen(Place::Mid, "0:50")));
    assert_eq!(whereabouts(&trackers[1]), Some(seen(Place::Base, "0:00")));
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

    assert_eq!(whereabouts(&trackers[0]), Some(seen(Place::Bot, "1:30")));
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

#[tokio::test]
async fn without_positions_the_jungler_is_who_holds_smite() {
    let drake = objective_kill("DragonKill", 400.0, ENEMY_JUNGLER);
    let tracker = tracker(TrackerPoll::at(405.0).event(drake).no_positions()).await;

    assert_eq!(whereabouts(&tracker), Some(seen(Place::Drake, "0:05")));
}

// The jungler cannot cross the map in under 40 s, so the far side lane is safe until then.
#[tokio::test]
async fn a_jungler_seen_bot_leaves_top_safe_for_a_while() {
    let kill = champion_kill(400.0, ENEMY_JUNGLER, ALLY_BOT, &[]);
    let trackers = trackers(vec![
        TrackerPoll::at(410.0).event(kill.clone()),
        TrackerPoll::at(440.0).event(kill),
    ])
    .await;

    let top_safe = SafeLane {
        lane: Place::Top,
        ends_in: "0:30".to_string(),
    };
    assert_eq!(
        trackers[0].jungler.as_ref().unwrap().safe_lane,
        Some(top_safe)
    );
    assert_eq!(trackers[1].jungler.as_ref().unwrap().safe_lane, None);
}

#[tokio::test]
async fn a_jungler_seen_mid_leaves_no_lane_safe() {
    let kill = champion_kill(400.0, ENEMY_JUNGLER, ME, &[]);
    let tracker = tracker(TrackerPoll::at(405.0).event(kill)).await;

    assert_eq!(tracker.jungler.unwrap().safe_lane, None);
}

// A jungler whose CS and level stand still is not clearing camps: ganking, or in base.
#[tokio::test]
async fn a_jungler_not_farming_shows_for_how_long() {
    let trackers = trackers(vec![
        TrackerPoll::at(600.0),
        TrackerPoll::at(610.0).farm(ENEMY_JUNGLER, 60, 6),
        TrackerPoll::at(699.0).farm(ENEMY_JUNGLER, 60, 6),
        TrackerPoll::at(700.0).farm(ENEMY_JUNGLER, 60, 6),
        TrackerPoll::at(701.0).farm(ENEMY_JUNGLER, 60, 7),
    ])
    .await;

    assert_eq!(trackers[2].jungler, None);
    let not_farming = trackers[3].jungler.as_ref().unwrap();
    assert_eq!(not_farming.no_farm_for.as_deref(), Some("1:30"));
    assert_eq!(not_farming.whereabouts, None);
    // A level is farm too, and CS moves only in steps of 10.
    assert_eq!(trackers[4].jungler, None);
}

#[tokio::test]
async fn a_lane_opponent_buying_is_in_base() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0).holding(ENEMY_MID, &[LONG_SWORD]),
        TrackerPoll::at(301.0).holding(ENEMY_MID, &[LONG_SWORD, SERRATED_DIRK]),
        TrackerPoll::at(335.0).holding(ENEMY_MID, &[LONG_SWORD, SERRATED_DIRK]),
        TrackerPoll::at(336.0).holding(ENEMY_MID, &[LONG_SWORD, SERRATED_DIRK]),
    ])
    .await;

    assert_eq!(trackers[0].recalls, []);
    assert_eq!(trackers[1].recalls, zed_recall("0:35", &["Serrated Dirk"]));
    assert_eq!(trackers[2].recalls, zed_recall("0:01", &["Serrated Dirk"]));
    // By then they are walking back into lane.
    assert_eq!(trackers[3].recalls, []);
}

#[tokio::test]
async fn a_lane_opponent_with_teleport_is_flagged() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0).teleport(ENEMY_MID),
        TrackerPoll::at(301.0)
            .teleport(ENEMY_MID)
            .holding(ENEMY_MID, &[SERRATED_DIRK]),
    ])
    .await;

    assert!(trackers[1].recalls[0].teleport);
}

#[tokio::test]
async fn purchases_on_one_recall_join() {
    let trackers = trackers(vec![
        TrackerPoll::at(300.0),
        TrackerPoll::at(301.0).holding(ENEMY_MID, &[SERRATED_DIRK]),
        TrackerPoll::at(305.0).holding(ENEMY_MID, &[SERRATED_DIRK, HEALTH_POTION]),
    ])
    .await;

    let both = zed_recall("0:31", &["Serrated Dirk", "Health Potion"]);
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

// Manamune turns into Muramana in lane, so the new item proves no Recall.
#[tokio::test]
async fn an_item_upgrading_in_lane_is_no_recall() {
    let trackers = trackers(vec![
        TrackerPoll::at(900.0).holding(ENEMY_MID, &[MANAMUNE]),
        TrackerPoll::at(901.0).holding(ENEMY_MID, &[MURAMANA]),
    ])
    .await;

    assert_eq!(trackers[1].recalls, []);
}

// Biscuit Delivery hands out biscuits in lane every few minutes.
#[tokio::test]
async fn a_rune_gift_is_no_recall() {
    let trackers = trackers(vec![
        TrackerPoll::at(239.0),
        TrackerPoll::at(240.0).holding(ENEMY_MID, &[BISCUIT]),
    ])
    .await;

    assert_eq!(trackers[1].recalls, []);
}

// A recall channels for 8 s, so someone in a kill 5 s ago cannot have reached the shop.
#[tokio::test]
async fn an_item_right_after_a_kill_is_no_recall() {
    let kill = champion_kill(300.0, ENEMY_MID, ME, &[]);
    let trackers = trackers(vec![
        TrackerPoll::at(304.0).event(kill.clone()),
        TrackerPoll::at(305.0)
            .event(kill)
            .holding(ENEMY_MID, &[SERRATED_DIRK]),
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
            lead_ends_in: "0:10".to_string(),
            objective: Some(WindowObjective {
                objective: Place::Baron,
                spawns_in: None,
            }),
        }),
        jungler: Some(JunglerSighting {
            champion: "Vi".to_string(),
            whereabouts: Some(Whereabouts::Seen {
                place: Place::Drake,
                ago: "0:05".to_string(),
            }),
            safe_lane: Some(SafeLane {
                lane: Place::Top,
                ends_in: "0:35".to_string(),
            }),
            no_farm_for: None,
        }),
        recalls: zed_recall("0:30", &["Serrated Dirk"]),
    };

    assert_eq!(
        serde_json::to_value(&tracker).unwrap(),
        json!({
            "headcount": {
                "allies": 5,
                "enemies": 4,
                "leadEndsIn": "0:10",
                "objective": { "objective": "baron", "spawnsIn": null },
            },
            "jungler": {
                "champion": "Vi",
                "whereabouts": { "state": "seen", "place": "drake", "ago": "0:05" },
                "safeLane": { "lane": "top", "endsIn": "0:35" },
                "noFarmFor": null,
            },
            "recalls": [{
                "champion": "Zed",
                "backIn": "0:30",
                "teleport": false,
                "bought": ["Serrated Dirk"],
            }],
        })
    );
    let dead = Whereabouts::Dead {
        back_in: "0:10".to_string(),
    };
    assert_eq!(
        serde_json::to_value(&dead).unwrap(),
        json!({ "state": "dead", "backIn": "0:10" })
    );
}
