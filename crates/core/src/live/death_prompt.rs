//! What death Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::death::Death;
use super::item_prompt::{clock, describe, item_lines};
use super::kill_score::KillScore;
use super::recommended_item::RecommendedItem;
use crate::advice::{instructions, Language};
use crate::benchmarks::ItemCandidate;

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here the player has just died to an enemy champion: `advice` says why and what to \
change, and names one of the Item options when the input gives them: the first unless \
the enemy team's items, levels or the kill score call for another. `item` is that item's \
id, or null when the input gives none. Never advise an item outside the options. When \
the death teaches nothing, as in a fair trade, `advice` and `item` are null."
);

pub(super) fn input(
    language: Language,
    death: &Death<'_>,
    options: &[ItemCandidate<'_>],
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
    if let Some(score) = KillScore::of(data, death.player) {
        let _ = writeln!(lines, "Kills: {score}");
    }
    for assister in death.assisters() {
        let _ = writeln!(lines, "Assister: {}", describe(assister));
    }
    for enemy in death.enemies() {
        let _ = writeln!(lines, "Enemy: {}", describe(enemy));
    }
    item_lines(&mut lines, language, options, recommended);
    lines
}
