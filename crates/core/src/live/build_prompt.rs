//! The Build lines of a prompt whose answer may change the Next item.

use std::fmt::Write;

use super::build::Build;
use super::payload::Player;
use crate::advice::Language;

/// What a Trigger whose input lists Item candidates adds to its instructions.
macro_rules! build_instructions {
    () => {
        "When the input lists Item candidates, `next_item` may put the id of one in place \
of the Next item, when the enemies' damage, items and levels, the kill score or the \
player's deaths call for it, and `reason` says why, at most 120 characters, in the input's \
language. Otherwise `next_item` and `reason` are null. `advice` never names an item."
    };
}
pub(super) use build_instructions;

/// The Build left to buy, the Next item, and the Item candidates that may replace it.
pub(super) fn build_lines(lines: &mut String, build: &Build, player: &Player, language: Language) {
    let left = build.left(&player.item_ids());
    let names: Vec<String> = left
        .iter()
        .map(|item| build.name_of(item.item, language))
        .collect();
    if names.is_empty() {
        let _ = writeln!(lines, "Build: none");
    } else {
        let _ = writeln!(lines, "Build, in order: {}", names.join(", "));
    }
    let Some(open) = build.open(player) else {
        if let Some(next) = names.first() {
            let _ = writeln!(lines, "Next item: {next}, started");
        }
        let _ = writeln!(lines, "Item candidates: none");
        return;
    };
    let _ = writeln!(
        lines,
        "Next item: {}, not started",
        build.name_of(open.next.item, language)
    );
    if open.replacements.is_empty() {
        let _ = writeln!(lines, "Item candidates: none");
    }
    for candidate in open.replacements {
        let later = if left.iter().any(|item| item.item == candidate.item) {
            ", later in the Build"
        } else {
            ""
        };
        let _ = writeln!(
            lines,
            "Item candidate: {} {}, bought in {:.0}% of games, first at minute {:.0}{later}",
            candidate.item,
            build.name_of(candidate.item, language),
            candidate.share * 100.0,
            candidate.minute
        );
    }
}

/// The champion's share of magic damage, empty when the table has none for it.
pub(super) fn magic_damage(build: &Build, player: &Player) -> String {
    build.magic_share(player).map_or_else(String::new, |share| {
        format!(", magic damage {:.0}%", share * 100.0)
    })
}

/// What a Trigger whose input may list Swap candidates adds to its instructions.
macro_rules! swap_instructions {
    () => {
        " When the input lists Swap candidates, `swap` may sell one Swap sell option for one \
Swap candidate, `sell` and `buy` by id, when the enemies' damage, items and levels, the kill \
score or the player's deaths call for it, and its `reason` says why, at most 120 \
characters, in the input's language. Otherwise `swap` is null."
    };
}
pub(super) use swap_instructions;

/// Once the inventory is full, the items a Swap may sell and the Item candidates it may
/// buy. Nothing while a slot is free.
pub(super) fn swap_lines(lines: &mut String, build: &Build, player: &Player, language: Language) {
    let Some(options) = build.swap_options(player) else {
        return;
    };
    if options.sells.is_empty() || options.buys.is_empty() {
        let _ = writeln!(lines, "Inventory full, Swap candidates: none");
        return;
    }
    let _ = writeln!(lines, "Inventory full");
    for item in options.sells {
        let _ = writeln!(lines, "Swap sell option: {} {}", item.id, item.shown_name());
    }
    for candidate in options.buys {
        let _ = writeln!(
            lines,
            "Swap candidate: {} {}, bought in {:.0}% of games, first at minute {:.0}",
            candidate.item,
            build.name_of(candidate.item, language),
            candidate.share * 100.0,
            candidate.minute
        );
    }
}
