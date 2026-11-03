//! What Back to base Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::back_to_base::BackToBase;
use super::item_prompt::{clock, describe, item_lines};
use super::payload::{AllGameData, Item, Player};
use super::recommended_item::RecommendedItem;
use crate::advice::{instructions, Language};
use crate::benchmarks::ItemCandidate;

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here the player is back to base and has just bought items: `advice` says what to buy \
next. `item` is the id of the one item candidate named in `advice`, or null when none \
fits. Say to buy it now when the player's gold affords it, otherwise to build toward it. \
When nothing has changed since an earlier recommended item, both `advice` and `item` \
are null."
);

/// The game at a Back to base.
pub(super) struct Visit<'a> {
    pub(super) data: &'a AllGameData,
    pub(super) player: &'a Player,
    pub(super) back: &'a BackToBase,
}

pub(super) fn input(
    language: Language,
    visit: &Visit<'_>,
    candidates: &[ItemCandidate<'_>],
    recommended: &[RecommendedItem<'_>],
) -> String {
    let Visit { data, player, back } = visit;
    let mut lines = String::new();
    let _ = writeln!(lines, "Language: {}", language.name());
    let _ = writeln!(
        lines,
        "Moment: the player is back to base and has just bought items"
    );
    let _ = writeln!(lines, "Game time: {}", clock(data.game_time()));
    let _ = writeln!(
        lines,
        "Player: {}, gold {:.0}",
        describe(player),
        data.current_gold()
    );
    let bought: Vec<String> = player
        .items
        .iter()
        .filter(|item| back.bought.contains(&item.id))
        .map(Item::shown_name)
        .collect();
    let _ = writeln!(lines, "Bought in this visit: {}", bought.join(", "));
    for enemy in data.enemies_of(player) {
        let _ = writeln!(lines, "Enemy: {}", describe(enemy));
    }
    item_lines(&mut lines, language, candidates, recommended);
    lines
}
