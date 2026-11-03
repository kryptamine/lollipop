use std::collections::BTreeMap;

use super::benchmark_file::{ItemCandidatesEntry, ItemNameEntry};
use super::item_catalog::ItemCatalog;

/// The name of every item some list holds, and of no other.
pub(super) fn item_names(
    candidates: &[ItemCandidatesEntry],
    catalog: &ItemCatalog,
) -> BTreeMap<u32, ItemNameEntry> {
    candidates
        .iter()
        .flat_map(|entry| [&entry.physical_heavy, &entry.mixed, &entry.magic_heavy])
        .flatten()
        .filter_map(|candidate| {
            let name = catalog.names.get(&candidate.item)?;
            Some((candidate.item, name.clone()))
        })
        .collect()
}
