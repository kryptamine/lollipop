//! The prompt lines every Trigger that may name an item shares. Never a Riot ID or a
//! summoner name.

use std::fmt::Write;

use super::payload::{Item, Player};
use super::recommended_item::RecommendedItem;
use crate::advice::Language;
use crate::benchmarks::{ItemCandidate, ItemName};

/// The Item options, best first, then the items earlier Advice recommended.
pub(super) fn item_lines(
    lines: &mut String,
    language: Language,
    options: &[ItemCandidate<'_>],
    recommended: &[RecommendedItem<'_>],
) {
    if options.is_empty() {
        let _ = writeln!(lines, "Item options: none");
    }
    for (rank, option) in options.iter().enumerate() {
        let _ = writeln!(
            lines,
            "Item option {}: {} {}, bought in {:.0}% of games, first at minute {:.0}",
            rank + 1,
            option.item,
            item_name(option.item, option.name, language),
            option.share * 100.0,
            option.minute
        );
    }
    for earlier in recommended {
        let bought = if earlier.bought {
            "bought"
        } else {
            "not bought"
        };
        let _ = writeln!(
            lines,
            "Earlier recommended: {}, {bought}",
            item_name(earlier.item, earlier.name, language)
        );
    }
}

/// Champion, position, level, KDA and items.
pub(super) fn describe(player: &Player) -> String {
    let mut parts = vec![player
        .champion_name
        .as_deref()
        .or_else(|| player.cell_champion())
        .unwrap_or("unknown champion")
        .to_string()];
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

fn item_name(item: u32, name: Option<&ItemName>, language: Language) -> String {
    name.map_or_else(
        || format!("item {item}"),
        |name| name.of(language).to_string(),
    )
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
