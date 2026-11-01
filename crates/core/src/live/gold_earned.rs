use std::collections::HashMap;

const STARTING_GOLD: f64 = 500.0;
/// Gold can only be accounted for from a game's first minute; past it, a count would be short.
const LATEST_START: f64 = 60.0;

/// How many of each item id a player holds. `slot` changes on a reorder, so it says nothing.
pub(super) type ItemCounts = HashMap<u32, u32>;

/// Counts one game's Gold earned from `currentGold`: see ADR 0002.
#[derive(Default)]
pub(super) struct GoldEarned {
    latest: Option<Latest>,
}

struct Latest {
    game_time: f64,
    /// `None` when the game's first snapshot came too late to count it.
    count: Option<Count>,
}

struct Count {
    total: f64,
    gold: f64,
    items: ItemCounts,
}

impl GoldEarned {
    /// A failed poll must not end the count: a late restart could never recover it.
    pub(super) fn end_game(&mut self) {
        self.latest = None;
    }

    pub(super) fn observe(&mut self, game_time: f64, gold: f64, items: ItemCounts) -> Option<u32> {
        // Only a new game turns the clock back, however the last one ended.
        let same_game = self.latest.take().filter(|l| l.game_time <= game_time);
        let count = match same_game {
            Some(latest) => latest.count.map(|count| count.advanced(gold, items)),
            None => (game_time < LATEST_START).then_some(Count {
                total: STARTING_GOLD,
                gold,
                items,
            }),
        };
        let total = count.as_ref().map(Count::whole_gold);
        self.latest = Some(Latest { game_time, count });
        total
    }
}

impl Count {
    fn advanced(self, gold: f64, items: ItemCounts) -> Self {
        // A fall in count is a leaving too: one potion sold from a stack refunds gold.
        let item_left = self
            .items
            .iter()
            .any(|(id, count)| items.get(id).is_none_or(|now| now < count));
        let gained = if item_left {
            0.0
        } else {
            (gold - self.gold).max(0.0)
        };
        Self {
            total: self.total + gained,
            gold,
            items,
        }
    }

    #[allow(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "no std conversion from f64; the total is whole, positive and far below u32::MAX"
    )]
    const fn whole_gold(&self) -> u32 {
        self.total.floor() as u32
    }
}
