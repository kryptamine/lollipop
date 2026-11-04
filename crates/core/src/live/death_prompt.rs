//! What death Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::death::Death;
use super::kill_score::KillScore;
use super::prompt_lines::{clock, describe};
use crate::advice::{instructions, Language};

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here the player has just died to an enemy champion: `advice` says why and what to \
change. Never name an item to buy: the overlay shows the build. When the death teaches \
nothing, as in a fair trade, `advice` is null."
);

pub(super) fn input(language: Language, death: &Death<'_>) -> String {
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
    lines
}
