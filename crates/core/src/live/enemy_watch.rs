use std::collections::HashMap;

use super::payload::{AllGameData, Player};
use super::recall::Recall;
use super::watched_enemy::WatchedEnemy;

/// Every enemy the overlay keeps track of, by the name they go by.
#[derive(Default)]
pub(super) struct EnemyWatch {
    enemies: HashMap<String, WatchedEnemy>,
}

impl EnemyWatch {
    pub(super) fn observe(&mut self, data: &AllGameData, player: &Player) {
        for enemy in data.enemies_of(player) {
            let Some(identity) = enemy.identity() else {
                continue;
            };
            if let Some(watched) = self.enemies.get_mut(identity) {
                watched.observe(data, enemy);
                continue;
            }
            let watched = WatchedEnemy::new(data.game_time(), enemy);
            self.enemies.insert(identity.to_string(), watched);
        }
    }

    pub(super) fn recall_of(&self, enemy: &Player) -> Option<&Recall> {
        self.watched(enemy)?.recall.as_ref()
    }

    /// When the enemy was last known to be in base: a Recall, or a respawn.
    pub(super) fn in_base_at(&self, enemy: &Player) -> Option<f64> {
        let watched = self.watched(enemy)?;
        let recalled = watched.recall.as_ref().map(|recall| recall.at);
        recalled
            .into_iter()
            .chain(watched.respawned_at)
            .reduce(f64::max)
    }

    pub(super) fn farmed_at(&self, enemy: &Player) -> Option<f64> {
        Some(self.watched(enemy)?.farmed_at)
    }

    fn watched(&self, enemy: &Player) -> Option<&WatchedEnemy> {
        self.enemies.get(enemy.identity()?)
    }
}
