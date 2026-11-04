//! What death Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::build::Build;
use super::build_prompt::{
    build_instructions, build_lines, elixir_instructions, elixir_lines, magic_damage,
    swap_instructions, swap_lines,
};
use super::death::Death;
use super::event_name::EventName;
use super::kill_score::KillScore;
use super::prompt_lines::{champion, clock, describe};
use crate::advice::{instructions, Language};

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here the player has just died to an enemy champion: `advice` says why and what to \
change. Never name an item to buy: the overlay shows the build. When the death teaches \
nothing, as in a fair trade, `advice` is null. ",
    build_instructions!(),
    swap_instructions!(),
    elixir_instructions!()
);

pub(super) fn input(language: Language, death: &Death<'_>, build: &Build) -> String {
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
    let _ = writeln!(
        lines,
        "Killer: {}{}",
        describe(death.killer),
        magic_damage(build, death.killer)
    );
    let _ = writeln!(
        lines,
        "Deaths to this killer this game: {}",
        death.kills_by_killer()
    );
    if let Some(score) = KillScore::of(data, death.player) {
        let _ = writeln!(lines, "Kills: {score}");
    }
    for assister in death.assisters() {
        let _ = writeln!(
            lines,
            "Assister: {}{}",
            describe(assister),
            magic_damage(build, assister)
        );
    }
    for enemy in death.enemies() {
        let _ = writeln!(
            lines,
            "Enemy: {}{}",
            describe(enemy),
            magic_damage(build, enemy)
        );
    }
    death_lines(&mut lines, death, build);
    build_lines(&mut lines, build, death.player, language);
    swap_lines(&mut lines, build, death.player, language);
    elixir_lines(&mut lines, build, death.player);
    lines
}

/// Every death of the player to an enemy champion this game, this one included.
fn death_lines(lines: &mut String, death: &Death<'_>, build: &Build) {
    let (data, player) = (death.data, death.player);
    let deaths = data
        .events_named(EventName::ChampionKill)
        .filter_map(|kill| Death::of(data, player, kill));
    for earlier in deaths {
        let at = earlier
            .kill
            .time()
            .map_or_else(String::new, |at| format!(" at {}", clock(at)));
        let assisters: Vec<String> = earlier
            .assisters()
            .map(|assister| format!("{}{}", champion(assister), magic_damage(build, assister)))
            .collect();
        let assisted = if assisters.is_empty() {
            String::new()
        } else {
            format!(", assisted by {}", assisters.join(" and "))
        };
        let _ = writeln!(
            lines,
            "Death{at}: to {}{}{assisted}",
            champion(earlier.killer),
            magic_damage(build, earlier.killer)
        );
    }
}
