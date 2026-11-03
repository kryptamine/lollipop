//! What death Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::death::Death;
use super::item_prompt::{clock, describe, item_lines};
use super::recommended_item::RecommendedItem;
use crate::advice::{instructions, Language};
use crate::benchmarks::ItemCandidate;

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here the player has just died to an enemy champion: `advice` says why and what to \
change. `item` is the id of the one item candidate that answers this death, named in \
`advice`, or null when none does: not every death needs an item. Advise an earlier \
recommended item again only if the game still calls for it."
);

pub(super) fn input(
    language: Language,
    death: &Death<'_>,
    candidates: &[ItemCandidate<'_>],
    recommended: &[RecommendedItem<'_>],
) -> String {
    let data = death.data;
    let mut lines = String::new();
    let _ = writeln!(lines, "Language: {}", language.name());
    let _ = writeln!(
        lines,
        "Moment: the player was just killed by an enemy champion"
    );
    let _ = writeln!(lines, "Game time: {}", clock(data.game_time()));
    let _ = writeln!(
        lines,
        "Player: {}, gold {:.0}",
        describe(death.player),
        data.current_gold()
    );
    let _ = writeln!(lines, "Killer: {}", describe(death.killer));
    let _ = writeln!(
        lines,
        "Deaths to this killer this game: {}",
        death.kills_by_killer()
    );
    for assister in death.assisters() {
        let _ = writeln!(lines, "Assister: {}", describe(assister));
    }
    for enemy in death.enemies() {
        let _ = writeln!(lines, "Enemy: {}", describe(enemy));
    }
    item_lines(&mut lines, language, candidates, recommended);
    lines
}
