//! What Turning point Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::game_facts::GameFacts;
use super::kill_score::KillScore;
use super::objective_score::ObjectiveScore;
use super::payload::{AllGameData, Player};
use super::prompt_lines::{clock, describe, objectives, standing};
use super::turning_point::TurningPoint;
use crate::advice::Language;

/// This Trigger's part of the instructions; its Proposals add theirs.
pub(super) const INSTRUCTIONS: &str = "Here a moment has just changed the game plan: `advice` is one macro idea for the next \
few minutes: which objective to play for and what to prioritise. Where the player is on \
the map is unknown, so never send them to a lane or to a named ally. Plan around the \
objectives that are up or spawn soon, by the times the input gives. When nothing is worth saying, or an earlier Advice already says it, `advice` is null.";

/// The game at a Turning point, with the texts of the Advice this game showed before.
pub(super) struct Turn<'a> {
    pub(super) data: &'a AllGameData,
    pub(super) player: &'a Player,
    pub(super) point: &'a TurningPoint,
    pub(super) facts: &'a GameFacts<'a>,
    pub(super) earlier: &'a [String],
}

pub(super) fn input(language: Language, turn: &Turn<'_>) -> String {
    let Turn {
        data,
        player,
        point,
        facts,
        earlier,
    } = turn;
    let own = player.team();
    let mut lines = String::new();
    let _ = writeln!(lines, "Language: {}", language.name());
    let _ = writeln!(lines, "Moment: {}", point.moment.describe());
    if let Some(team) = point.taken_by {
        let side = if Some(team) == own {
            "the player's team"
        } else {
            "the enemy team"
        };
        let _ = writeln!(lines, "Taken by: {side}");
    }
    let _ = writeln!(lines, "Game time: {}", clock(data.game_time()));
    let _ = writeln!(
        lines,
        "Player: {}, gold {:.0}",
        describe(player),
        data.current_gold()
    );
    if let Some(line) = standing(facts) {
        let _ = writeln!(lines, "{line}");
    }
    let team = player.team();
    for other in data.players().iter().filter(|p| !std::ptr::eq(*p, *player)) {
        let side = if other.team() == team {
            "Ally"
        } else {
            "Enemy"
        };
        let _ = writeln!(lines, "{side}: {}", describe(other));
    }
    if let Some(score) = KillScore::of(data, player) {
        let _ = writeln!(lines, "Kills: {score}");
    }
    if let Some(own) = own {
        let _ = writeln!(
            lines,
            "Player's team has taken: {}",
            ObjectiveScore::of(data, own)
        );
        let _ = writeln!(
            lines,
            "Enemy team has taken: {}",
            ObjectiveScore::of(data, own.other())
        );
    }
    if let Some(line) = objectives(facts, data.game_time()) {
        let _ = writeln!(lines, "{line}");
    }
    for text in *earlier {
        let _ = writeln!(lines, "Earlier Advice: {text}");
    }
    lines
}
