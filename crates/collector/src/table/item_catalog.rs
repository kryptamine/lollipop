use std::collections::{BTreeMap, HashMap};

use super::benchmark_file::ItemNameEntry;
use super::item_class::ItemClass;

/// What Data Dragon says of the items, at the newest patch of the Patch window.
pub(super) struct ItemCatalog {
    pub(super) version: String,
    pub(super) classes: HashMap<u32, ItemClass>,
    /// The transforms each purchasable item becomes with no purchase, such as Seraph's
    /// Embrace of Archangel's Staff, in id order.
    pub(super) owned_as: HashMap<u32, Vec<u32>>,
    /// Every component down each item's `from` tree, in id order.
    pub(super) builds_from: HashMap<u32, Vec<u32>>,
    pub(super) names: HashMap<u32, ItemNameEntry>,
    /// Data Dragon's tags of each item, such as `Armor` or `SpellDamage`.
    pub(super) tags: HashMap<u32, Vec<String>>,
    /// Every boots of the Rift past the first tier, in id order.
    pub(super) boots: Vec<u32>,
}

impl ItemCatalog {
    pub(super) fn completed(&self) -> Vec<u32> {
        let mut completed: Vec<u32> = self
            .classes
            .iter()
            .filter(|(_, class)| **class == ItemClass::Completed)
            .map(|(&id, _)| id)
            .collect();
        completed.sort_unstable();
        completed
    }

    /// Only completed items and boots: no other item reaches a prompt.
    pub(super) fn item_tags(&self) -> BTreeMap<u32, Vec<String>> {
        self.completed()
            .into_iter()
            .chain(self.boots.iter().copied())
            .filter_map(|id| Some((id, self.tags.get(&id)?.clone())))
            .collect()
    }
}
