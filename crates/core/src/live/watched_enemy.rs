use super::fighting::fought_within;
use super::gold_earned::ItemCounts;
use super::payload::{AllGameData, Player};
use super::purchases::bought_since;
use super::recall::Recall;

// Players buy in base at the start and as they respawn, which says nothing new.
const STARTING_PURCHASES_UNTIL: f64 = 90.0;
const SHOPPING_AFTER_RESPAWN: f64 = 30.0;
// Purchases this close to the first are the same Recall.
const ONE_RECALL: f64 = 20.0;
// A recall channels for 8 s, so a kill this recent rules out the shop.
const RECALL_CHANNEL: f64 = 8.0;
// Jungle camps first spawn here, so no farm before it means nothing.
const CAMPS_SPAWN: f64 = 90.0;

/// What the overlay keeps of one enemy from one poll to the next.
pub(super) struct WatchedEnemy {
    items: ItemCounts,
    cs: u32,
    level: u32,
    dead: bool,
    pub(super) respawned_at: Option<f64>,
    /// When their CS or level last rose, or they respawned.
    pub(super) farmed_at: f64,
    /// `None` once they have died since.
    pub(super) recall: Option<Recall>,
}

impl WatchedEnemy {
    pub(super) fn new(now: f64, enemy: &Player) -> Self {
        Self {
            items: enemy.item_counts(),
            cs: enemy.scores.creep_score,
            level: enemy.level,
            dead: enemy.is_dead,
            respawned_at: None,
            farmed_at: now.max(CAMPS_SPAWN),
            recall: None,
        }
    }

    pub(super) fn observe(&mut self, data: &AllGameData, enemy: &Player) {
        let now = data.game_time();
        let bought = bought_since(&self.items, enemy);
        self.items = enemy.item_counts();
        if enemy.scores.creep_score > self.cs || enemy.level > self.level {
            self.farmed_at = now;
        }
        self.cs = enemy.scores.creep_score;
        self.level = enemy.level;
        if enemy.is_dead {
            self.dead = true;
            self.recall = None;
            return;
        }
        if std::mem::take(&mut self.dead) {
            self.respawned_at = Some(now);
            self.farmed_at = now;
        }
        let shopping = now < STARTING_PURCHASES_UNTIL
            || self
                .respawned_at
                .is_some_and(|at| now - at < SHOPPING_AFTER_RESPAWN);
        if bought.is_empty() || shopping || fought_within(data, enemy, RECALL_CHANNEL) {
            return;
        }
        match &mut self.recall {
            Some(recall) if now - recall.at < ONE_RECALL => recall.add(bought),
            _ => self.recall = Some(Recall { at: now, bought }),
        }
    }
}
