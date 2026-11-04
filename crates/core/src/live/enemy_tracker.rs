use serde::Serialize;

use super::clock_text::countdown;
use super::enemy_watch::EnemyWatch;
use super::headcount::{headcount_of, Headcount};
use super::jungler_sighting::{sighting_of, JunglerSighting};
use super::lane_opponent::lane_of;
use super::payload::{AllGameData, Player};
use super::shown_recall::ShownRecall;
use super::spawn::Spawn;

// About the walk from base back to a lane.
const WALK_BACK: f64 = 35.0;

/// What the overlay works out of the enemies from what every player can see: the
/// scoreboard and the kill feed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnemyTracker {
    pub headcount: Option<Headcount>,
    pub jungler: Option<JunglerSighting>,
    /// The Lane opponents in base.
    pub recalls: Vec<ShownRecall>,
}

impl EnemyTracker {
    pub(super) fn at(
        watch: &EnemyWatch,
        data: &AllGameData,
        player: &Player,
        spawns: &[Spawn],
    ) -> Self {
        Self {
            headcount: headcount_of(data, player, spawns),
            jungler: sighting_of(data, player, watch),
            recalls: recalls_of(watch, data, player),
        }
    }
}

fn recalls_of(watch: &EnemyWatch, data: &AllGameData, player: &Player) -> Vec<ShownRecall> {
    let Some(lane) = lane_of(data, player) else {
        return Vec::new();
    };
    let now = data.game_time();
    lane.opponents
        .iter()
        .filter_map(|opponent| {
            let recall = watch.recall_of(opponent)?;
            let passed = now - recall.at;
            if passed >= WALK_BACK {
                return None;
            }
            Some(ShownRecall {
                champion: opponent.champion_name.clone()?,
                back_in: countdown(WALK_BACK - passed),
                // Unleashed Teleport's raw name is not in any recording.
                teleport: opponent.has_summoner("SummonerTeleport"),
                bought: recall.bought.clone(),
            })
        })
        .collect()
}
