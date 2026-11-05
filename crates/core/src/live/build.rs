use super::build_change::BuildChange;
use super::build_item::BuildItem;
use super::counter_item::CounterItem;
use super::elixir::{Elixir, ELIXIR_COST};
use super::gold_earned::ItemCounts;
use super::lead::Lead;
use super::open_next::OpenNext;
use super::payload::{AllGameData, Player};
use super::shown_swap::ShownSwap;
use super::swap::Swap;
use super::swap_options::SwapOptions;
use crate::advice::{Language, ProposedChange, ProposedSwap};
use crate::benchmarks::{BenchmarkTable, ItemCandidate, ItemTable, ProfileBucket};
use crate::game_inputs::{Patch, Role};

// Completed items in a Build, beside its boots.
const BUILD_ITEMS: usize = 5;

/// One game's Build, less the items another one took the place of.
#[derive(Default)]
pub(super) struct Build {
    /// Empty when the game's table is missing or Stale: see ADR 0004.
    table: ItemTable,
    /// The player's champion and role, to read the candidates of another bucket by.
    seat: Option<(String, Role)>,
    bucket: Option<ProfileBucket>,
    /// The Item candidates, largest share first.
    candidates: Vec<ItemCandidate>,
    items: Vec<ItemCandidate>,
    /// Dropped for good: finishing one later drops no other item.
    dropped: Vec<ItemCandidate>,
    /// The Counter items put in, which no other item takes the place of.
    counters: Vec<CounterItem>,
    /// The completed items and boots held at the last poll, `None` before the first.
    finished: Option<Vec<u32>>,
    lead: Option<Lead>,
    /// The items held when the lead was last checked.
    held: Option<ItemCounts>,
    change: Option<BuildChange>,
    swap: Option<Swap>,
    /// The model's Elixir pick, which holds for the rest of the game.
    elixir: Option<Elixir>,
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
        let seat = player.cell_champion().map(str::to_string).zip(role);
        let bucket = table.bucket_of(enemy_items(data, player));
        let (candidates, usual) = match (&seat, bucket) {
            (Some((champion, role)), Some(bucket)) => (
                table.candidates(champion, *role, bucket),
                table.usual_build(champion, *role, bucket).to_vec(),
            ),
            _ => (Vec::new(), Vec::new()),
        };
        let mut build = Self {
            items: first_build(&candidates, &usual),
            table,
            seat,
            bucket,
            candidates,
            ..Self::default()
        };
        build.add_counters(data, player);
        build
    }

    /// Builds again from the candidates of the bucket the enemies' items put them in.
    /// Held items and a started Next item stay; nothing else does: see ADR 0019.
    pub(super) fn follow_enemies(&mut self, data: &AllGameData, player: &Player) {
        let Some((champion, role)) = self.seat.clone() else {
            return;
        };
        let bucket = self.table.bucket_of(enemy_items(data, player));
        let Some(new) = bucket.filter(|_| bucket != self.bucket) else {
            return;
        };
        let owned = player.item_ids();
        let started = self.started_next(player).cloned();
        self.bucket = bucket;
        self.candidates = self.table.candidates(&champion, role, new);
        let usual = self.table.usual_build(&champion, role, new);
        self.items = first_build(&self.candidates, usual);
        self.dropped.clear();
        self.counters.clear();
        for item in self.table.finished(&owned) {
            self.finish(item, &owned);
        }
        if let Some(next) = started {
            self.keep_next(next, &owned);
        }
        self.add_counters(data, player);
    }

    pub(super) const fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// The items left to buy in build order, the Next item first, no more than the free
    /// slots. Items held at the first poll, as when the app starts mid-game, drop nothing.
    pub(super) fn at(&mut self, player: &Player, language: Language) -> Vec<BuildItem> {
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
            .map(|candidate| self.shown(candidate.item, language))
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
        if lead.started(&next.builds_from, &player.item_counts()) || self.is_counter(next.item) {
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
        let shod = self.shod(owned);
        self.items
            .iter()
            .filter(|candidate| !(candidate.held(owned) || candidate.boots && shod))
            .take(self.table.free_slots(owned))
            .collect()
    }

    /// Once the inventory is full, the held completed items a Swap may sell, never boots,
    /// and the Item candidates it may buy: none held, and boots only with none held.
    pub(super) fn swap_options<'a>(&'a self, player: &'a Player) -> Option<SwapOptions<'a>> {
        let owned = player.item_ids();
        if self.candidates.is_empty() || self.table.free_slots(&owned) > 0 {
            return None;
        }
        let finished = self.table.finished(&owned);
        let mut sells: Vec<_> = player
            .items
            .iter()
            .filter(|item| finished.contains(&item.id) && !self.table.is_boots(item.id))
            .collect();
        sells.sort_by_key(|item| item.id);
        sells.dedup_by_key(|item| item.id);
        let shod = self.shod(&owned);
        let buys = self
            .candidates
            .iter()
            .filter(|candidate| !(candidate.held(&owned) || candidate.boots && shod))
            .collect();
        Some(SwapOptions { sells, buys })
    }

    /// Holds `proposed` as the one Swap, in place of the one before. Anything
    /// `swap_options` does not offer is ignored: see ADR 0004.
    pub(super) fn swap(&mut self, proposed: ProposedSwap, player: &Player) {
        let Some(options) = self.swap_options(player) else {
            return;
        };
        if !options.sells.iter().any(|item| item.id == proposed.sell) {
            return;
        }
        let Some(buy) = options
            .buys
            .iter()
            .find(|candidate| candidate.item == proposed.buy)
            .map(|candidate| (*candidate).clone())
        else {
            return;
        };
        self.swap = Some(Swap {
            sell: proposed.sell,
            buy,
            reason: proposed.reason,
        });
    }

    /// The Swap while the inventory is full. It ends once its item to buy is held, or
    /// the inventory is full again without its item to sell.
    pub(super) fn swap_at(&mut self, player: &Player, language: Language) -> Option<ShownSwap> {
        let owned = player.item_ids();
        let swap = self.swap.as_ref()?;
        let full = self.table.free_slots(&owned) == 0;
        let sold = !owned.contains(&swap.sell);
        if swap.buy.held(&owned) || full && sold {
            self.swap = None;
        }
        let swap = self.swap.as_ref().filter(|_| full)?;
        Some(ShownSwap {
            sell: self.shown(swap.sell, language),
            buy: self.shown(swap.buy.item, language),
            reason: swap.reason.clone(),
        })
    }

    /// With the inventory full and no Elixir picked yet, a model may pick one.
    pub(super) fn elixir_open(&self, player: &Player) -> bool {
        self.elixir.is_none() && self.table.free_slots(&player.item_ids()) == 0
    }

    /// Holds the model's pick when it is open and names an Elixir; anything else is
    /// ignored, and code picks instead.
    pub(super) fn pick_elixir(&mut self, item: u32, player: &Player) {
        if self.elixir_open(player) {
            self.elixir = Elixir::of_item(item);
        }
    }

    /// The Elixir to buy with a full inventory and the gold for it: the model's pick,
    /// else Sorcery or Wrath by the champion's magic share, and none without one.
    pub(super) fn elixir_at(
        &self,
        player: &Player,
        gold: f64,
        language: Language,
    ) -> Option<BuildItem> {
        if gold < ELIXIR_COST || self.table.free_slots(&player.item_ids()) > 0 {
            return None;
        }
        let elixir = self
            .elixir
            .or_else(|| self.magic_share(player).map(Elixir::by_magic_share))?;
        Some(self.shown(elixir.item(), language))
    }

    /// The name in `language`, or the id in a table built without names.
    pub(super) fn name_of(&self, item: u32, language: Language) -> String {
        self.table.name_of(item).map_or_else(
            || format!("item {item}"),
            |name| name.of(language).to_string(),
        )
    }

    pub(super) fn tags_of(&self, item: u32) -> &[String] {
        self.table.tags_of(item)
    }

    pub(super) fn magic_share(&self, player: &Player) -> Option<f64> {
        self.table.magic_share(player.cell_champion()?)
    }

    /// An item off the Build takes the place of the first one left of its kind, boots
    /// or completed, which drops for good. One that counters a Threat takes the place
    /// of the Counter item put in for it, and no other item takes that place.
    fn finish(&mut self, item: u32, owned: &[u32]) {
        let mut built = self.items.iter().chain(&self.dropped);
        if built.any(|candidate| candidate.is(item)) {
            return;
        }
        let countered = self
            .counters
            .iter()
            .find(|counter| counter.kind.contains(&item))
            .map(|counter| counter.item);
        let boots = self.table.is_boots(item);
        let replaced = self.items.iter().position(|candidate| {
            !candidate.held(owned)
                && countered.map_or_else(
                    || candidate.boots == boots && !self.is_counter(candidate.item),
                    |counter| candidate.item == counter,
                )
        });
        if let Some(index) = replaced {
            self.dropped.push(self.items.remove(index));
        }
    }

    /// A Counter item for each Threat the enemies are heavy in, unless the Build or the
    /// player holds one already: see ADR 0023.
    fn add_counters(&mut self, data: &AllGameData, player: &Player) {
        let enemies: Vec<&str> = data
            .enemies_of(player)
            .filter_map(Player::cell_champion)
            .collect();
        let share = self.magic_share(player);
        let wanted: Vec<Vec<u32>> = self
            .table
            .counters_against(&enemies, share)
            .into_iter()
            .map(<[u32]>::to_vec)
            .collect();
        let owned = player.item_ids();
        for kind in wanted {
            self.add_counter_from(kind, &owned);
        }
    }

    /// Puts the candidate of `kind` the player has a component of, else the one with the
    /// largest share, after the Next item by minute.
    fn add_counter_from(&mut self, kind: Vec<u32>, owned: &[u32]) {
        let countered = self.items.iter().any(|item| kind.contains(&item.item))
            || owned.iter().any(|item| kind.contains(item));
        if countered {
            return;
        }
        let mut open = self.candidates.iter().filter(|candidate| {
            !candidate.boots && kind.contains(&candidate.item) && !self.was_dropped(candidate.item)
        });
        let started = |candidate: &&ItemCandidate| {
            candidate
                .builds_from
                .iter()
                .any(|component| owned.contains(component))
        };
        let Some(counter) = open.clone().find(started).or_else(|| open.next()).cloned() else {
            return;
        };
        let Some(at) = self.counter_slot(counter.minute, owned) else {
            return;
        };
        self.counters.push(CounterItem {
            item: counter.item,
            kind,
        });
        self.items.insert(at, counter);
    }

    /// Where a Counter item bought at `minute` goes, after the Next item. In a Build of
    /// five completed items, the last one left that is not a Counter item drops for it;
    /// `None` with no such item, or no Next item.
    fn counter_slot(&mut self, minute: f64, owned: &[u32]) -> Option<usize> {
        let next = self.left(owned).first()?.item;
        let next_at = self.items.iter().position(|item| item.item == next)?;
        let completed = self.items.iter().filter(|item| !item.boots).count();
        if completed >= BUILD_ITEMS {
            let last = self
                .items
                .iter()
                .rposition(|item| !item.boots && !item.held(owned) && !self.is_counter(item.item));
            let last = last.filter(|&last| last > next_at)?;
            self.dropped.push(self.items.remove(last));
        }
        let at = self
            .items
            .iter()
            .enumerate()
            .skip(next_at + 1)
            .find(|(_, item)| item.minute > minute)
            .map_or(self.items.len(), |(at, _)| at);
        Some(at)
    }

    fn is_counter(&self, item: u32) -> bool {
        self.counters.iter().any(|counter| counter.item == item)
    }

    fn started_next(&self, player: &Player) -> Option<&ItemCandidate> {
        let lead = self.lead.as_ref()?;
        let next = self.items.iter().find(|item| item.item == lead.item)?;
        lead.started(&next.builds_from, &player.item_counts())
            .then_some(next)
    }

    /// Puts the started Next item first among those left, the last item left of its
    /// kind making room when the Build is full of that kind.
    fn keep_next(&mut self, next: ItemCandidate, owned: &[u32]) {
        self.dropped.retain(|dropped| dropped.item != next.item);
        if let Some(index) = self.items.iter().position(|item| item.item == next.item) {
            self.items.remove(index);
        } else {
            let cap = if next.boots { 1 } else { BUILD_ITEMS };
            let kind = self.items.iter().filter(|item| item.boots == next.boots);
            if kind.count() >= cap {
                let last = self
                    .items
                    .iter()
                    .rposition(|item| item.boots == next.boots && !item.held(owned));
                if let Some(index) = last {
                    self.dropped.push(self.items.remove(index));
                }
            }
        }
        let first_left = self
            .items
            .iter()
            .position(|item| !item.held(owned))
            .unwrap_or(self.items.len());
        self.items.insert(first_left, next);
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

    fn shown(&self, item: u32, language: Language) -> BuildItem {
        BuildItem {
            item,
            icon: self.table.icon_url(item),
            name: self
                .table
                .name_of(item)
                .map(|name| name.of(language).to_string()),
        }
    }

    /// A player holds one pair of boots, whichever it is.
    fn shod(&self, owned: &[u32]) -> bool {
        owned.iter().any(|&item| self.table.is_boots(item))
    }

    fn was_dropped(&self, item: u32) -> bool {
        self.dropped.iter().any(|dropped| dropped.item == item)
    }
}

fn enemy_items<'a>(
    data: &'a AllGameData,
    player: &'a Player,
) -> impl Iterator<Item = (&'a str, Vec<u32>)> {
    data.enemies_of(player)
        .filter_map(|enemy| Some((enemy.cell_champion()?, enemy.item_ids())))
}

/// The boots with the largest share and five completed items, by median minute: those of
/// the usual build, else in a table built before it those with the largest shares.
fn first_build(candidates: &[ItemCandidate], usual: &[u32]) -> Vec<ItemCandidate> {
    let boots = candidates
        .iter()
        .filter(|candidate| candidate.boots)
        .take(1);
    let completed: Vec<&ItemCandidate> = if usual.is_empty() {
        candidates
            .iter()
            .filter(|candidate| !candidate.boots)
            .take(BUILD_ITEMS)
            .collect()
    } else {
        usual
            .iter()
            .filter_map(|&item| {
                candidates
                    .iter()
                    .find(|candidate| candidate.item == item && !candidate.boots)
            })
            .take(BUILD_ITEMS)
            .collect()
    };
    let mut picked: Vec<ItemCandidate> = boots.chain(completed).cloned().collect();
    // The sort is stable, so a tie in minutes goes to the larger share.
    picked.sort_by(|a, b| a.minute.total_cmp(&b.minute));
    picked
}
