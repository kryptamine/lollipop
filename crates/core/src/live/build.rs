use super::build_change::BuildChange;
use super::build_item::BuildItem;
use super::gold_earned::ItemCounts;
use super::lead::Lead;
use super::open_next::OpenNext;
use super::payload::{AllGameData, Player};
use crate::advice::{Language, ProposedChange};
use crate::benchmarks::{BenchmarkTable, ItemCandidate, ItemTable};
use crate::game_inputs::{Patch, Role};

// Completed items in a Build, beside its boots.
const BUILD_ITEMS: usize = 5;

/// One game's Build, less the items another one took the place of.
#[derive(Default)]
pub(super) struct Build {
    /// Empty when the game's table is missing or Stale: see ADR 0004.
    table: ItemTable,
    /// The Item candidates, largest share first.
    candidates: Vec<ItemCandidate>,
    items: Vec<ItemCandidate>,
    /// Dropped for good: finishing one later drops no other item.
    dropped: Vec<ItemCandidate>,
    /// The completed items and boots held at the last poll, `None` before the first.
    finished: Option<Vec<u32>>,
    lead: Option<Lead>,
    /// The items held when the lead was last checked.
    held: Option<ItemCounts>,
    change: Option<BuildChange>,
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
        let candidates = match (role, player.cell_champion()) {
            (Some(role), Some(champion)) => {
                let enemies = data.enemies_of(player).filter_map(Player::cell_champion);
                table.candidates(champion, role, enemies)
            }
            _ => Vec::new(),
        };
        Self {
            items: first_build(&candidates),
            table,
            candidates,
            ..Self::default()
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
        self.track_lead(player);
        self.left(&owned)
            .into_iter()
            .map(|candidate| BuildItem {
                item: candidate.item,
                icon: self.table.icon_url(candidate.item),
            })
            .collect()
    }

    /// The reason of the last Build change while it shows, from the game's start on.
    pub(super) fn change_at(&mut self, game_time: f64, game_start: Option<f64>) -> Option<String> {
        let showing = self.change.as_mut()?.showing_at(game_time, game_start);
        if !showing {
            self.change = None;
        }
        self.change.as_ref().map(|change| change.reason.clone())
    }

    /// Notes the Next item. A component bought in the visit that finished the last one
    /// starts it too, so it counts from the items held at the last check.
    pub(super) fn track_lead(&mut self, player: &Player) {
        let held = player.item_counts();
        let before = self.held.replace(held.clone());
        let next = self.left(&player.item_ids()).first().map(|next| next.item);
        if self.lead.as_ref().map(|lead| lead.item) != next {
            let held_then = before.unwrap_or(held);
            self.lead = next.map(|item| Lead::new(item, held_then));
        }
    }

    /// Items of its own kind may take the Next item's place, boots for boots, but none
    /// held or dropped. `None` once the player started it.
    pub(super) fn open(&self, player: &Player) -> Option<OpenNext<'_>> {
        let lead = self.lead.as_ref()?;
        let next = self.items.iter().find(|item| item.item == lead.item)?;
        if lead.started(&next.builds_from, &player.item_counts()) {
            return None;
        }
        let owned = player.item_ids();
        let replacements = self
            .candidates
            .iter()
            .filter(|candidate| candidate.boots == next.boots && candidate.item != next.item)
            .filter(|candidate| !candidate.held(&owned) && !self.was_dropped(candidate.item))
            .collect();
        Some(OpenNext { next, replacements })
    }

    /// Puts `proposed` in place of the Next item, which drops. An item taken from later
    /// in the Build leaves a gap, which the next candidate of the kind by share fills.
    /// Anything `open` does not offer is ignored: see ADR 0004.
    pub(super) fn change(
        &mut self,
        proposed: ProposedChange,
        player: &Player,
        game_time: Option<f64>,
    ) {
        self.track_lead(player);
        let Some(open) = self.open(player) else {
            return;
        };
        let next = open.next.item;
        if !open
            .replacements
            .iter()
            .any(|item| item.item == proposed.item)
        {
            return;
        }
        let moved = self
            .items
            .iter()
            .position(|item| item.item == proposed.item);
        let replacement = match moved {
            Some(index) => self.items.remove(index),
            None => match self
                .candidates
                .iter()
                .find(|item| item.item == proposed.item)
            {
                Some(candidate) => candidate.clone(),
                None => return,
            },
        };
        let Some(slot) = self.items.iter_mut().find(|item| item.item == next) else {
            return;
        };
        let replaced = std::mem::replace(slot, replacement);
        let boots = replaced.boots;
        self.dropped.push(replaced);
        if moved.is_some() {
            self.refill(boots, &player.item_ids());
        }
        self.change = Some(BuildChange::new(proposed.reason, game_time));
    }

    /// The items left to buy, as [`Self::at`] shows them.
    pub(super) fn left(&self, owned: &[u32]) -> Vec<&ItemCandidate> {
        // A player holds one pair of boots, whichever it is.
        let shod = owned.iter().any(|&item| self.table.is_boots(item));
        self.items
            .iter()
            .filter(|candidate| !(candidate.held(owned) || candidate.boots && shod))
            .take(self.table.free_slots(owned))
            .collect()
    }

    /// The name in `language`, or the id in a table built without names.
    pub(super) fn name_of(&self, item: u32, language: Language) -> String {
        self.table.name_of(item).map_or_else(
            || format!("item {item}"),
            |name| name.of(language).to_string(),
        )
    }

    pub(super) fn magic_share(&self, player: &Player) -> Option<f64> {
        self.table.magic_share(player.cell_champion()?)
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

    fn refill(&mut self, boots: bool, owned: &[u32]) {
        let taken = |candidate: &ItemCandidate| {
            candidate.held(owned)
                || self
                    .items
                    .iter()
                    .chain(&self.dropped)
                    .any(|built| built.item == candidate.item)
        };
        let next = self
            .candidates
            .iter()
            .find(|candidate| candidate.boots == boots && !taken(candidate))
            .cloned();
        self.items.extend(next);
    }

    fn was_dropped(&self, item: u32) -> bool {
        self.dropped.iter().any(|dropped| dropped.item == item)
    }
}

/// The boots with the largest share and the five completed items with the largest
/// shares, by median minute.
fn first_build(candidates: &[ItemCandidate]) -> Vec<ItemCandidate> {
    let boots = candidates
        .iter()
        .filter(|candidate| candidate.boots)
        .take(1);
    let completed = candidates
        .iter()
        .filter(|candidate| !candidate.boots)
        .take(BUILD_ITEMS);
    let mut picked: Vec<ItemCandidate> = boots.chain(completed).cloned().collect();
    // The sort is stable, so a tie in minutes goes to the larger share.
    picked.sort_by(|a, b| a.minute.total_cmp(&b.minute));
    picked
}
