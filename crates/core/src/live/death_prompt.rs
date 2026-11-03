//! What death Advice is asked with. Never a Riot ID or a summoner name.

use std::fmt::Write;

use super::death::Death;
use super::payload::Player;
use super::recommended_item::RecommendedItem;
use crate::advice::{instructions, Language};
use crate::benchmarks::{ItemCandidate, ItemName};

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here the player has just died to an enemy champion: `advice` says why and what to \
change. `item` is the id of the one item candidate that answers this death, named in \
`advice`, or null when none does: not every death needs an item. Advise an earlier \
recommended item again only if the game still calls for it."
);

pub(super) fn input(
    language: Language,
    death: &Death<'_>,
    candidates: &[ItemCandidate<'_>],
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
    for assister in death.assisters() {
        let _ = writeln!(lines, "Assister: {}", describe(assister));
    }
    for enemy in death.enemies() {
        let _ = writeln!(lines, "Enemy: {}", describe(enemy));
    }
    if candidates.is_empty() {
        let _ = writeln!(lines, "Item candidates: none");
    } else {
        let _ = writeln!(lines, "Item candidates:");
    }
    for candidate in candidates {
        let _ = writeln!(
            lines,
            "- {} {}: bought in {:.0}% of games, first at minute {:.0}",
            candidate.item,
            item_name(candidate.item, candidate.name, language),
            candidate.share * 100.0,
            candidate.minute
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
    lines
}

/// Champion, position, level, KDA and items.
fn describe(player: &Player) -> String {
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
    let items: Vec<String> = player
        .items
        .iter()
        .map(|item| {
            item.display_name
                .clone()
                .unwrap_or_else(|| format!("item {}", item.id))
        })
        .collect();
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
fn clock(game_time: f64) -> String {
    let seconds = game_time.max(0.0) as u64;
    format!("{}:{:02}", seconds / 60, seconds % 60)
}
