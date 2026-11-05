use super::item_table::ItemTable;

/// An enemy's magic share: the champion's usual share, weighed as one item, averaged with
/// the completed items held that deal damage. Tank and utility items say nothing.
pub(super) fn enemy_magic_share(table: &ItemTable, usual: Option<f64>, items: &[u32]) -> f64 {
    let (sum, weight) = table
        .finished(items)
        .into_iter()
        .filter(|&item| !table.is_boots(item))
        .filter_map(|item| item_magic_share(table.tags_of(item)))
        .fold((usual.unwrap_or(0.0), 1.0), |(sum, weight), share| {
            (sum + share, weight + 1.0)
        });
    sum / weight
}

fn item_magic_share(tags: &[String]) -> Option<f64> {
    let has = |tag: &str| tags.iter().any(|held| held == tag);
    match (has("SpellDamage"), has("Damage")) {
        (true, true) => Some(0.5),
        (true, false) => Some(1.0),
        (false, true) => Some(0.0),
        (false, false) => None,
    }
}
