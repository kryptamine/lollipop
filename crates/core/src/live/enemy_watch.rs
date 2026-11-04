use std::collections::HashMap;

use super::gold_earned::ItemCounts;
use super::payload::{AllGameData, Player};
use super::purchases::bought_since;
use super::recall::Recall;

// Players buy in base at the start and as they respawn, which says nothing new.
const STARTING_PURCHASES_UNTIL: f64 = 90.0;
const SHOPPING_AFTER_RESPAWN: f64 = 30.0;
// Purchases this close to the first are the same trip to base.
const ONE_TRIP: f64 = 20.0;

/// What the overlay keeps of each enemy from one poll to the next.
#[derive(Default)]
pub(super) struct EnemyWatch {
    enemies: HashMap<String, Watched>,
}

struct Watched {
    items: ItemCounts,
    dead: bool,
    respawned_at: Option<f64>,
    recall: Option<Recall>,
}

impl EnemyWatch {
    pub(super) fn observe(&mut self, data: &AllGameData, player: &Player) {
        let now = data.game_time();
        for enemy in data.enemies_of(player) {
            let Some(key) = enemy.key() else {
                continue;
            };
            if let Some(watched) = self.enemies.get_mut(key) {
                watched.observe(now, enemy);
                continue;
            }
            let watched = Watched {
                items: enemy.item_counts(),
                dead: enemy.is_dead,
                respawned_at: None,
                recall: None,
            };
            self.enemies.insert(key.to_string(), watched);
        }
    }

    /// `None` once the enemy has died since.
    pub(super) fn recall_of(&self, enemy: &Player) -> Option<&Recall> {
        self.watched(enemy)?.recall.as_ref()
    }

    /// When the enemy was last known to be in base: a trip there, or a respawn.
    pub(super) fn in_base_at(&self, enemy: &Player) -> Option<f64> {
        let watched = self.watched(enemy)?;
        let recalled = watched.recall.as_ref().map(|recall| recall.at);
        recalled
            .into_iter()
            .chain(watched.respawned_at)
            .reduce(f64::max)
    }

    fn watched(&self, enemy: &Player) -> Option<&Watched> {
        self.enemies.get(enemy.key()?)
    }
}

impl Watched {
    fn observe(&mut self, now: f64, enemy: &Player) {
        let bought = bought_since(&self.items, enemy);
        self.items = enemy.item_counts();
        if enemy.is_dead {
            self.dead = true;
            self.recall = None;
            return;
        }
        if std::mem::take(&mut self.dead) {
            self.respawned_at = Some(now);
        }
        let shopping = now < STARTING_PURCHASES_UNTIL
            || self
                .respawned_at
                .is_some_and(|at| now - at < SHOPPING_AFTER_RESPAWN);
        if bought.is_empty() || shopping {
            return;
        }
        match &mut self.recall {
            Some(recall) if now - recall.at < ONE_TRIP => recall.add(bought),
            _ => self.recall = Some(Recall { at: now, bought }),
        }
    }
}
