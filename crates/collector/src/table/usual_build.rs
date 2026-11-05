use super::benchmark_file::CandidateEntry;
use super::first_purchase::FirstPurchase;
use super::item_catalog::ItemCatalog;
use super::item_class::ItemClass;

// Completed items in a Build, beside its boots.
const BUILD_ITEMS: usize = 5;

/// The completed candidates games buy together, most bought first. Each pick narrows the
/// games to those holding it, so an item bought instead of an earlier pick drops out;
/// once that would leave under `min_games`, the rest come from the last games kept.
pub(super) fn usual_build(
    games: &[&[FirstPurchase]],
    list: &[CandidateEntry],
    catalog: &ItemCatalog,
    min_games: usize,
) -> Vec<u32> {
    let completed: Vec<u32> = list
        .iter()
        .map(|candidate| candidate.item)
        .filter(|item| catalog.classes.get(item) == Some(&ItemClass::Completed))
        .collect();
    let mut kept: Vec<&[FirstPurchase]> = games.to_vec();
    let mut picked: Vec<u32> = Vec::new();
    while picked.len() < BUILD_ITEMS {
        let holding = |item: u32| kept.iter().filter(|game| bought(game, item)).count();
        // `max_by_key` keeps the last of a tie, so walking back gives a tie to the larger share.
        let best = completed
            .iter()
            .rev()
            .filter(|item| !picked.contains(item))
            .map(|&item| (holding(item), item))
            .max_by_key(|(games, _)| *games);
        let Some((games, item)) = best.filter(|(games, _)| *games > 0) else {
            break;
        };
        picked.push(item);
        if games >= min_games {
            kept.retain(|game| bought(game, item));
        }
    }
    picked
}

fn bought(game: &[FirstPurchase], item: u32) -> bool {
    game.iter().any(|purchase| purchase.item == item)
}
