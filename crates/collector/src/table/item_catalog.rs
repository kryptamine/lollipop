use std::collections::HashMap;

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
    /// Every boots of the Rift past the first tier, in id order.
    pub(super) boots: Vec<u32>,
}
