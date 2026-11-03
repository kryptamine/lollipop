//! What Turning point Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::item_prompt::{clock, describe};
use super::objective_score::ObjectiveScore;
use super::payload::{AllGameData, Player};
use super::team::Team;
use super::turning_point::TurningPoint;
use crate::advice::{instructions, Language};

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here a moment has just changed the game plan: `advice` is one macro idea for the next \
few minutes, where to be and what to prioritise. `item` is always null. When nothing is \
worth saying, or an earlier Advice already says it, `advice` is null."
);

/// The game at a Turning point, with the texts of the Advice this game showed before.
pub(super) struct Turn<'a> {
    pub(super) data: &'a AllGameData,
    pub(super) player: &'a Player,
    pub(super) point: &'a TurningPoint,
    pub(super) earlier: &'a [String],
}

pub(super) fn input(language: Language, turn: &Turn<'_>) -> String {
    let Turn {
        data,
        player,
        point,
        earlier,
    } = turn;
    let own = player.team().and_then(Team::of_player);
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
    let team = player.team();
    for other in data.players().iter().filter(|p| !std::ptr::eq(*p, *player)) {
        let side = if other.team() == team {
            "Ally"
        } else {
            "Enemy"
        };
        let _ = writeln!(lines, "{side}: {}", describe(other));
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
    for text in *earlier {
        let _ = writeln!(lines, "Earlier Advice: {text}");
    }
    lines
}
