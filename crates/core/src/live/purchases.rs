use super::gold_earned::ItemCounts;
use super::payload::{Item, Player};

// Items that arrive away from the shop: an upgrade such as Muramana or a finished
// support quest, a used Stopwatch, or a rune's gift such as a biscuit or free boots.
const NOT_FROM_THE_SHOP: [u32; 14] = [
    2010, 2421, 2422, 3040, 3042, 3121, 3513, 3866, 3867, 3869, 3870, 3871, 3876, 3877,
];

/// The names of the items `player` gained since `before` that only the shop sells.
pub(super) fn bought_since(before: &ItemCounts, player: &Player) -> Vec<String> {
    let now = player.item_counts();
    let mut bought: Vec<&Item> = Vec::new();
    for item in &player.items {
        let more = now.get(&item.id) > before.get(&item.id);
        let new = !bought.iter().any(|seen| seen.id == item.id);
        if more && new && !NOT_FROM_THE_SHOP.contains(&item.id) {
            bought.push(item);
        }
    }
    bought.iter().map(|item| item.shown_name()).collect()
}
