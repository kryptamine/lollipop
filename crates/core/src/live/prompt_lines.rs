//! The prompt lines the death and Turning point Triggers share. Never a Riot ID or a
//! summoner name.

use super::payload::{Item, Player};

/// Champion, position, level, KDA and items.
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
    let items: Vec<String> = player.items.iter().map(Item::shown_name).collect();
    if items.is_empty() {
        parts.push("no items".to_string());
    } else {
        parts.push(format!("items {}", items.join(", ")));
    }
    parts.join(", ")
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
