use super::gold_earned::ItemCounts;
use super::payload::{AllGameData, Player};

// Starting items are bought before this `gameTime`.
const EARLIEST: f64 = 90.0;
// The shop a respawn opens on, which Advice on death covers.
const AFTER_RESPAWN: f64 = 15.0;
// The rest of one visit's purchases, so a visit fires once.
const AFTER_BACK_TO_BASE: f64 = 30.0;
// Drops wards, potions and trinket swaps.
const LEAST_SPENT: f64 = 300.0;

/// The first purchase of a Back to base.
pub(super) struct BackToBase {
    pub(super) game_time: f64,
    /// The items new in this purchase.
    pub(super) bought: Vec<u32>,
}

/// Watches one game's polls for a Back to base. A recall that buys nothing is not seen.
#[derive(Default)]
pub(super) struct BackToBaseWatch {
    last: Option<Seen>,
    was_dead: bool,
    respawned_at: Option<f64>,
    fired_at: Option<f64>,
}

struct Seen {
    gold: f64,
    items: ItemCounts,
}

impl BackToBaseWatch {
    /// A new item and gold down by at least 300 in the same poll, while alive.
    /// An undo raises gold, so it never fires.
    pub(super) fn seen(&mut self, data: &AllGameData, player: &Player) -> Option<BackToBase> {
        let now = data.game_time();
        let items = player.item_counts();
        let last = self.last.replace(Seen {
            gold: data.current_gold(),
            items: items.clone(),
        });
        if player.is_dead {
            self.was_dead = true;
            return None;
        }
        if std::mem::take(&mut self.was_dead) {
            self.respawned_at = Some(now);
        }
        let last = last?;
        let too_soon = now < EARLIEST
            || self
                .respawned_at
                .is_some_and(|at| now - at <= AFTER_RESPAWN)
            || self
                .fired_at
                .is_some_and(|at| now - at <= AFTER_BACK_TO_BASE);
        if too_soon || last.gold - data.current_gold() < LEAST_SPENT {
            return None;
        }
        let bought: Vec<u32> = items
            .iter()
            .filter(|&(item, &count)| count > last.items.get(item).copied().unwrap_or(0))
            .map(|(&item, _)| item)
            .collect();
        if bought.is_empty() {
            return None;
        }
        self.fired_at = Some(now);
        Some(BackToBase {
            game_time: now,
            bought,
        })
    }
}
