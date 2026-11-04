//! The prompt lines the death and Turning point Triggers share. Never a Riot ID or a
//! summoner name.

use super::game_facts::GameFacts;
use super::payload::{Item, Player};

// Objectives that stay up until taken. Void Grubs and the Herald leave on their own, at
// times the schedule does not hold, so only their spawn to come is told.
const STAY_UP: [&str; 3] = ["Drake", "Elder", "Baron"];

/// Champion, position, level, KDA, CS and items.
pub(super) fn describe(player: &Player) -> String {
    let mut parts = vec![champion(player).to_string()];
    parts.extend(player.position.clone());
    parts.push(format!("level {}", player.level));
    let scores = &player.scores;
    if let (Some(kills), Some(deaths), Some(assists)) =
        (scores.kills, scores.deaths, scores.assists)
    {
        parts.push(format!("KDA {kills}/{deaths}/{assists}"));
    }
    parts.push(format!("CS {}", scores.creep_score));
    let items: Vec<String> = player.items.iter().map(Item::shown_name).collect();
    if items.is_empty() {
        parts.push("no items".to_string());
    } else {
        parts.push(format!("items {}", items.join(", ")));
    }
    parts.join(", ")
}

/// The player against games of their Cell at this minute, `None` with no Benchmark.
pub(super) fn standing(facts: &GameFacts<'_>) -> Option<String> {
    let (snapshot, benchmarks) = (facts.snapshot, facts.benchmarks);
    let mut parts = Vec::new();
    if let Some(cs) = benchmarks.cs {
        parts.push(format!("CS {} against {cs:.0}", snapshot.cs));
    }
    if let Some((gold, benchmark)) = snapshot.gold_earned.zip(benchmarks.gold) {
        parts.push(format!("gold earned {gold} against {benchmark:.0}"));
    }
    if let Some(level) = benchmarks.level {
        parts.push(format!("level {} against {level:.1}", snapshot.level));
    }
    if let Some((own, usual)) = snapshot.scores.zip(benchmarks.scores) {
        parts.push(format!(
            "KDA {}/{}/{} against {:.1}/{:.1}/{:.1}",
            own.kills, own.deaths, own.assists, usual.kills, usual.deaths, usual.assists
        ));
    }
    (!parts.is_empty()).then(|| {
        format!(
            "Player against games of the same champion, role and tier at this minute: {}",
            parts.join(", ")
        )
    })
}

/// Each objective's next spawn, or that it is up, `None` with no Objective schedule.
pub(super) fn objectives(facts: &GameFacts<'_>, game_time: f64) -> Option<String> {
    let parts: Vec<String> = facts
        .spawns
        .iter()
        .filter_map(|spawn| {
            if spawn.at > game_time {
                Some(format!(
                    "{} in {}",
                    spawn.objective,
                    clock(spawn.at - game_time)
                ))
            } else {
                STAY_UP
                    .contains(&spawn.objective)
                    .then(|| format!("{} up now", spawn.objective))
            }
        })
        .collect();
    (!parts.is_empty()).then(|| format!("Objectives: {}", parts.join(", ")))
}

pub(super) fn champion(player: &Player) -> &str {
    player
        .champion_name
        .as_deref()
        .or_else(|| player.cell_champion())
        .unwrap_or("unknown champion")
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "game time is a few thousand seconds, never negative"
)]
pub(super) fn clock(game_time: f64) -> String {
    let seconds = game_time.max(0.0) as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
