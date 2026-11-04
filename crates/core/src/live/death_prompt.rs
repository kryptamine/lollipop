//! What death Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::build::Build;
use super::build_prompt::magic_damage;
use super::death::Death;
use super::event_name::EventName;
use super::game_facts::GameFacts;
use super::health_trail::HealthTrail;
use super::kill_score::KillScore;
use super::payload::ChampionStats;
use super::prompt_lines::{champion, clock, describe, objectives, standing};
use crate::advice::Language;

/// This Trigger's part of the instructions; its Proposals add theirs.
pub(super) const INSTRUCTIONS: &str =
    "Here the player has just died to an enemy champion: `advice` says why and what to \
change. Never name an item to buy: the overlay shows the build. The player's health \
before death tells a burst from a long fight, and their defences against the killer's \
damage say what they lacked. When the death teaches nothing, as in a fair trade, \
`advice` is null. ";

/// What death Advice reads beyond the death itself.
pub(super) struct Around<'a> {
    pub(super) build: &'a Build,
    pub(super) facts: &'a GameFacts<'a>,
    pub(super) health: &'a HealthTrail,
}

pub(super) fn input(language: Language, death: &Death<'_>, around: &Around<'_>) -> String {
    let Around {
        build,
        facts,
        health,
    } = *around;
    let data = death.data;
    let mut lines = String::new();
    let _ = writeln!(lines, "Language: {}", language.name());
    let _ = writeln!(
        lines,
        "Moment: the player was just killed by an enemy champion"
    );
    let _ = writeln!(lines, "Game time: {}", clock(data.game_time()));
    player_lines(&mut lines, death, facts, health);
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
    if let Some(line) = objectives(facts, data.game_time()) {
        let _ = writeln!(lines, "{line}");
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
    lines
}

/// The player as they died: against their Benchmarks, their defences and last health.
fn player_lines(
    lines: &mut String,
    death: &Death<'_>,
    facts: &GameFacts<'_>,
    health: &HealthTrail,
) {
    let data = death.data;
    let _ = writeln!(
        lines,
        "Player: {}, gold {:.0}",
        describe(death.player),
        data.current_gold()
    );
    if let Some(line) = standing(facts) {
        let _ = writeln!(lines, "{line}");
    }
    if let Some(stats) = data.champion_stats() {
        let _ = writeln!(lines, "Player defences: {}", defences(stats));
    }
    health_lines(lines, death, health);
    let _ = writeln!(
        lines,
        "Respawn in: {} s",
        death.player.respawn_timer.round()
    );
}

fn defences(stats: &ChampionStats) -> String {
    let parts = [
        ("max health", stats.max_health),
        ("armor", stats.armor),
        ("magic resist", stats.magic_resist),
    ];
    parts
        .iter()
        .filter_map(|(name, value)| value.map(|value| format!("{name} {value:.0}")))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The player's health over the last seconds before this death, nothing without it.
fn health_lines(lines: &mut String, death: &Death<'_>, health: &HealthTrail) {
    let Some(died_at) = death.kill.time() else {
        return;
    };
    let samples: Vec<String> = health
        .before(died_at)
        .map(|(at, share)| format!("{:.0}% at {}", share * 100.0, clock(at)))
        .collect();
    if !samples.is_empty() {
        let _ = writeln!(lines, "Player health before death: {}", samples.join(", "));
    }
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
