use super::payload::Player;
use super::recommended_item::RecommendedItem;
use super::shown_item::ShownItem;
use super::standing::Standing;
use crate::benchmarks::{BenchmarkTable, ItemCandidate, ItemTable};
use crate::game_inputs::{ClientInputs, GameInputs, Role};

/// One game's Item candidates and the items its Advice showed, shared by every
/// Trigger that may name an item.
#[derive(Default)]
pub(super) struct AdvisedItems {
    /// Empty when the game's table is missing or Stale, so no item can be advised.
    table: ItemTable,
    role: Option<Role>,
    shown: Vec<ShownItem>,
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

    /// The one candidate item Advice may name: the one most held components build into,
    /// then one Advice showed and the player has not left for another finished item,
    /// then the earliest by minute not skipped, then the latest skipped. `None` once
    /// the inventory is full.
    pub(super) fn to_build<'a>(
        &self,
        player: &Player,
        enemies: impl Iterator<Item = &'a Player>,
    ) -> Option<ItemCandidate<'_>> {
        let (Some(role), Some(champion)) = (self.role, player.cell_champion()) else {
            return None;
        };
        let owned = player.item_ids();
        if self.table.full(&owned) {
            return None;
        }
        let finished = self.table.finished(&owned);
        let enemies = enemies.filter_map(Player::cell_champion);
        let candidates = self.table.candidates(champion, role, enemies, &owned);
        let held = |candidate: &ItemCandidate<'_>| {
            candidate
                .builds_from
                .iter()
                .filter(|item| owned.contains(item))
                .count()
        };
        let standing = |candidate: &ItemCandidate<'_>| {
            let Some((index, shown)) = self
                .shown
                .iter()
                .enumerate()
                .find(|(_, shown)| shown.item == candidate.item)
            else {
                return Standing::NeverShown;
            };
            if shown.passed_over(&finished) {
                Standing::PassedOver
            } else {
                Standing::InProgress(index)
            }
        };
        // A skipped item sorts by its minute negated: late in a build, the latest one
        // left is the nearest to where the player is.
        let by_minute = |candidate: &ItemCandidate<'_>| {
            if candidate.skipped {
                -candidate.minute
            } else {
                candidate.minute
            }
        };
        // `min_by` keeps the first of equals, so a tie goes to the larger share.
        candidates.into_iter().min_by(|a, b| {
            held(b)
                .cmp(&held(a))
                .then_with(|| standing(a).cmp(&standing(b)))
                .then(a.skipped.cmp(&b.skipped))
                .then(by_minute(a).total_cmp(&by_minute(b)))
        })
    }

    /// Items earlier Advice in this game showed, and whether `player` bought them,
    /// less boots passed over for another pair.
    pub(super) fn recommended(&self, player: &Player) -> Vec<RecommendedItem<'_>> {
        let owned = player.item_ids();
        self.shown
            .iter()
            .map(|shown| shown.item)
            .filter(|&item| !self.table.another_pair(item, &owned))
            .map(|item| RecommendedItem {
                item,
                name: self.table.name_of(item),
                bought: owned.contains(&item),
            })
            .collect()
    }

    /// Whether every slot holds a completed item or boots.
    pub(super) fn full(&self, player: &Player) -> bool {
        self.table.full(&player.item_ids())
    }

    pub(super) fn icon_url(&self, item: u32) -> Option<String> {
        self.table.icon_url(item)
    }

    pub(super) fn show(&mut self, item: u32, player: &Player) {
        if !self.shown.iter().any(|shown| shown.item == item) {
            self.shown.push(ShownItem {
                item,
                finished_then: self.table.finished(&player.item_ids()),
            });
        }
    }
}
