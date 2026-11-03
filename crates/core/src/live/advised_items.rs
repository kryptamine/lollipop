use super::payload::Player;
use super::recommended_item::RecommendedItem;
use crate::benchmarks::{BenchmarkTable, ItemCandidate, ItemTable};
use crate::game_inputs::{ClientInputs, GameInputs, Role};

/// One game's Item candidates and the items its Advice showed, shared by every
/// Trigger that may name an item.
#[derive(Default)]
pub(super) struct AdvisedItems {
    /// Empty when the game's table is missing or Stale, so no item can be advised.
    table: ItemTable,
    role: Option<Role>,
    shown: Vec<u32>,
}

impl AdvisedItems {
    pub(super) fn start(table: Option<&mut BenchmarkTable>, inputs: &GameInputs) -> Self {
        let patch = match &inputs.client {
            ClientInputs::Read(client) => client.patch,
            ClientInputs::Unavailable => None,
        };
        let table = table
            .filter(|table| patch.is_some_and(|patch| table.serves(patch)))
            .map(BenchmarkTable::take_items)
            .unwrap_or_default();
        Self {
            table,
            role: inputs.role,
            shown: Vec::new(),
        }
    }

    /// The one candidate item Advice may name: an item Advice showed and the player has not
    /// bought, else the one most held components build into, else the earliest by minute.
    pub(super) fn to_build<'a>(
        &self,
        player: &Player,
        enemies: impl Iterator<Item = &'a Player>,
    ) -> Option<ItemCandidate<'_>> {
        let (Some(role), Some(champion)) = (self.role, player.cell_champion()) else {
            return None;
        };
        let owned = player.item_ids();
        let enemies = enemies.filter_map(Player::cell_champion);
        let mut candidates = self.table.candidates(champion, role, enemies, &owned);
        let shown = self.shown.iter().find_map(|&item| {
            candidates
                .iter()
                .position(|candidate| candidate.item == item)
        });
        if let Some(shown) = shown {
            return Some(candidates.swap_remove(shown));
        }
        let held = |candidate: &ItemCandidate<'_>| {
            candidate
                .builds_from
                .iter()
                .filter(|item| owned.contains(item))
                .count()
        };
        // `min_by` keeps the first of equals, so a tie goes to the larger share.
        candidates
            .into_iter()
            .min_by(|a, b| held(b).cmp(&held(a)).then(a.minute.total_cmp(&b.minute)))
    }

    /// Items earlier Advice in this game showed, and whether `player` bought them.
    pub(super) fn recommended(&self, player: &Player) -> Vec<RecommendedItem<'_>> {
        let owned = player.item_ids();
        self.shown
            .iter()
            .map(|&item| RecommendedItem {
                item,
                name: self.table.name_of(item),
                bought: owned.contains(&item),
            })
            .collect()
    }

    pub(super) fn icon_url(&self, item: u32) -> Option<String> {
        self.table.icon_url(item)
    }

    pub(super) fn show(&mut self, item: u32) {
        if !self.shown.contains(&item) {
            self.shown.push(item);
        }
    }
}
