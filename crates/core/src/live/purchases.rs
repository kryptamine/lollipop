use super::gold_earned::ItemCounts;
use super::payload::Player;

// Items that turn into another away from the shop, such as a stacked Tear, a finished
// support quest or a used Stopwatch. Their arrival proves no trip to base.
const FIELD_UPGRADES: [u32; 12] = [
    2421, 3040, 3042, 3121, 3513, 3866, 3867, 3869, 3870, 3871, 3876, 3877,
];

/// What `player` holds more of than `before`, by the client's name, once each: items only
/// the shop sells.
pub(super) fn bought_since(before: &ItemCounts, player: &Player) -> Vec<String> {
    let now = player.item_counts();
    let mut bought: Vec<u32> = Vec::new();
    let mut names = Vec::new();
    for item in &player.items {
        let more = now.get(&item.id) > before.get(&item.id);
        if more && !FIELD_UPGRADES.contains(&item.id) && !bought.contains(&item.id) {
            bought.push(item.id);
            names.push(item.shown_name());
        }
    }
    names
}
