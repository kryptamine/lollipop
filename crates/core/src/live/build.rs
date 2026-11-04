use super::build_item::BuildItem;
use super::payload::{AllGameData, Player};
use crate::benchmarks::{BenchmarkTable, ItemCandidate, ItemTable};
use crate::game_inputs::{Patch, Role};

/// One game's Build, less the items the player finished another one in place of.
#[derive(Default)]
pub(super) struct Build {
    /// Empty when the game's table is missing or Stale: see ADR 0004.
    table: ItemTable,
    items: Vec<ItemCandidate>,
    /// Dropped for good: finishing one later drops no other item.
    dropped: Vec<ItemCandidate>,
    /// The completed items and boots held at the last poll, `None` before the first.
    finished: Option<Vec<u32>>,
}

impl Build {
    pub(super) fn start(
        table: Option<&mut BenchmarkTable>,
        patch: Option<Patch>,
        role: Option<Role>,
        data: &AllGameData,
        player: &Player,
    ) -> Self {
        let Some(table) = table
            .filter(|table| patch.is_some_and(|patch| table.serves(patch)))
            .map(BenchmarkTable::take_items)
        else {
            return Self::default();
        };
        let items = match (role, player.cell_champion()) {
            (Some(role), Some(champion)) => {
                let enemies = data.enemies_of(player).filter_map(Player::cell_champion);
                table.build(champion, role, enemies)
            }
            _ => Vec::new(),
        };
        Self {
            table,
            items,
            dropped: Vec::new(),
            finished: None,
        }
    }

    pub(super) const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The items left to buy in build order, the Next item first, no more than the free
    /// slots. Items held at the first poll, as when the app starts mid-game, drop nothing.
    pub(super) fn at(&mut self, player: &Player) -> Vec<BuildItem> {
        let owned = player.item_ids();
        let finished = self.table.finished(&owned);
        let before = self.finished.replace(finished.clone());
        let new = finished
            .iter()
            .filter(|item| before.as_ref().is_some_and(|before| !before.contains(item)));
        for &item in new {
            self.finish(item, &owned);
        }
        // A player holds one pair of boots, whichever it is.
        let shod = owned.iter().any(|&item| self.table.is_boots(item));
        self.items
            .iter()
            .filter(|candidate| !(candidate.held(&owned) || candidate.boots && shod))
            .take(self.table.free_slots(&owned))
            .map(|candidate| BuildItem {
                item: candidate.item,
                icon: self.table.icon_url(candidate.item),
            })
            .collect()
    }

    /// An item off the Build takes the place of the first one left of its kind, boots
    /// or completed, which drops for good.
    fn finish(&mut self, item: u32, owned: &[u32]) {
        let mut built = self.items.iter().chain(&self.dropped);
        if built.any(|candidate| candidate.is(item)) {
            return;
        }
        let boots = self.table.is_boots(item);
        let replaced = self
            .items
            .iter()
            .position(|candidate| candidate.boots == boots && !candidate.held(owned));
        if let Some(index) = replaced {
            self.dropped.push(self.items.remove(index));
        }
    }
}
