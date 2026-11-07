//! What a Recap's Lessons are asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::payload::{AllGameData, Player};
use super::prompt_lines::{champion, clock, describe};
use super::recap::Recap;
use super::recap_minute::RecapMinute;
use crate::advice::Language;
use crate::game_inputs::Tier;

// Every fifth minute and the last: enough to show where a game slipped, short enough
// to keep a 40-minute game's input small.
const MINUTE_STEP: u32 = 5;

pub(super) const INSTRUCTIONS: &str =
    "You coach a League of Legends player whose game has just ended. Write at most three \
lessons for their next game as `lessons`, most costly first, each at most 160 characters, \
in the language the input names. Ground each one in this game: where CS or gold earned \
fell behind games of the same champion, role and tier, the deaths and who caused them, \
the stretches of slow farm, and what the enemies built. Never a tip that fits any game, \
such as to ward more or play safe. Fewer lessons beat a weak one, and an empty list is \
an answer.";

pub(super) fn input(
    language: Language,
    recap: &Recap,
    data: &AllGameData,
    tier: Option<Tier>,
) -> String {
    let mut lines = String::new();
    let _ = writeln!(lines, "Language: {}", language.name());
    let mut game = vec![recap.champion.clone()];
    game.extend(recap.role.map(|role| role.label().to_string()));
    game.extend(tier.map(|tier| tier.label().to_string()));
    game.push(format!("length {}", clock(recap.length)));
    game.extend(recap.outcome.map(|outcome| outcome.label().to_string()));
    let _ = writeln!(lines, "Game: {}", game.join(", "));
    let player = data.active_player();
    if let Some(player) = player {
        let _ = writeln!(lines, "Player at the end: {}", describe(player));
    }
    minute_lines(&mut lines, &recap.minutes);
    death_lines(&mut lines, data);
    for stretch in &recap.slow_farm {
        let _ = writeln!(
            lines,
            "Slow farm against games like it: {} to {}",
            clock(stretch.from),
            clock(stretch.to)
        );
    }
    let team = player.and_then(Player::team);
    for other in data.players() {
        if player.is_some_and(|player| std::ptr::eq(player, other)) {
            continue;
        }
        let side = if other.team() == team {
            "Ally"
        } else {
            "Enemy"
        };
        let _ = writeln!(lines, "{side}: {}", describe(other));
    }
    lines
}

fn minute_lines(lines: &mut String, minutes: &[RecapMinute]) {
    let last = minutes.last().map(|minute| minute.minute);
    for minute in minutes
        .iter()
        .filter(|minute| minute.minute % MINUTE_STEP == 0 || Some(minute.minute) == last)
    {
        let mut parts = vec![against("CS", minute.cs, minute.cs_benchmark)];
        if let Some(gold) = minute.gold_earned {
            parts.push(against("gold earned", gold, minute.gold_benchmark));
        }
        let _ = writeln!(lines, "Minute {}: {}", minute.minute, parts.join(", "));
    }
}

fn against(name: &str, value: u32, benchmark: Option<f64>) -> String {
    benchmark.map_or_else(
        || format!("{name} {value}"),
        |benchmark| format!("{name} {value} against {benchmark:.0}"),
    )
}

fn death_lines(lines: &mut String, data: &AllGameData) {
    let Some(player) = data.active_player() else {
        return;
    };
    for death in data.deaths_of(player) {
        let at = death
            .time()
            .map_or_else(String::new, |at| format!(" at {}", clock(at)));
        let killer = death
            .killer_name
            .as_deref()
            .and_then(|killer| data.player_named(killer))
            .map_or("a turret, minion or monster", champion);
        let _ = writeln!(lines, "Death{at}: to {killer}");
    }
}
