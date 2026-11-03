//! What loading screen Advice is asked with. Never a Riot ID, a summoner name or an
//! enemy's rank.

use std::fmt::Write;

use super::lane_opponent::Lane;
use super::payload::{AllGameData, FullRunes, Named, Player};
use crate::advice::{instructions, Language};
use crate::game_inputs::TierReading;

pub(super) const INSTRUCTIONS: &str = instructions!(
    "Here `advice` is a plan for the laning phase against the lane opponent, and `item` \
is always null."
);

/// The game's part of the input, read from the loading screen payload.
pub(super) fn game_lines(data: &AllGameData, player: &Player, lane: &Lane<'_>) -> String {
    let mut lines = String::new();
    let _ = writeln!(lines, "Player: {}", describe(player));
    if let Some(runes) = data.full_runes() {
        let _ = writeln!(lines, "Player runes: {}", full_runes(runes));
    }
    let _ = writeln!(lines, "Lane opponent: {}", champions(&lane.opponents));
    if let Some(partner) = lane.partner {
        let _ = writeln!(lines, "Lane partner: {}", champion(partner));
    }
    let team = player.team();
    for p in data.players().iter().filter(|p| !std::ptr::eq(*p, player)) {
        let side = if p.team() == team { "Ally" } else { "Enemy" };
        let _ = writeln!(lines, "{side}: {}", describe(p));
    }
    lines
}

pub(super) fn input(language: Language, tier: TierReading, game_lines: &str) -> String {
    let tier = match tier {
        TierReading::Ranked(tier) => tier.label(),
        TierReading::Unranked => "unranked",
        TierReading::Unknown => "unknown",
    };
    format!(
        "Language: {}\nMoment: loading screen, before the game starts\nPlayer tier: {tier}\n{game_lines}",
        language.name()
    )
}

fn describe(player: &Player) -> String {
    let mut parts = vec![champion(player).to_string()];
    parts.extend(player.position.clone());
    if let Some(runes) = &player.runes {
        let trees = [&runes.primary_rune_tree, &runes.secondary_rune_tree];
        parts.push(format!(
            "keystone {} ({})",
            name(runes.keystone.as_ref()),
            trees.map(|tree| name(tree.as_ref())).join(", ")
        ));
    }
    if let Some(spells) = &player.summoner_spells {
        let spells = [&spells.summoner_spell_one, &spells.summoner_spell_two];
        parts.push(format!(
            "spells {}",
            spells.map(|spell| name(spell.as_ref())).join(", ")
        ));
    }
    parts.join(", ")
}

fn full_runes(runes: &FullRunes) -> String {
    let general: Vec<&str> = runes.general_runes.iter().map(|r| name(Some(r))).collect();
    let shards: Vec<&str> = runes
        .stat_runes
        .iter()
        .filter_map(|shard| shard.raw_description.as_deref())
        .map(|shard| shard.trim_start_matches("perk_tooltip_"))
        .collect();
    format!(
        "keystone {}, {} and {}: {}; shards: {}",
        name(runes.keystone.as_ref()),
        name(runes.primary_rune_tree.as_ref()),
        name(runes.secondary_rune_tree.as_ref()),
        general.join(", "),
        shards.join(", ")
    )
}

fn champions(players: &[&Player]) -> String {
    players
        .iter()
        .map(|p| champion(p))
        .collect::<Vec<_>>()
        .join(" and ")
}

fn champion(player: &Player) -> &str {
    player
        .champion_name
        .as_deref()
        .or_else(|| player.cell_champion())
        .unwrap_or("unknown champion")
}

fn name(named: Option<&Named>) -> &str {
    named
        .and_then(|named| named.display_name.as_deref())
        .unwrap_or("unknown")
}
