use std::path::Path;

use serde::Deserialize;

use super::cell_benchmarks::CellBenchmarks;
use super::champion_key::champion_key;
use super::item_table::ItemTable;
use super::minute_entry::MinuteEntry;
use crate::game_inputs::{Patch, Role, Tier};

pub(crate) const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Deserialize)]
pub struct BenchmarkTable {
    schema_version: u32,
    max_patch: Option<Patch>,
    patch_window: Vec<Patch>,
    cells: Vec<CellEntry>,
    #[serde(flatten)]
    items: ItemTable,
}

#[derive(Debug, Deserialize)]
struct CellEntry {
    champion: String,
    role: String,
    tier: String,
    minutes: Vec<MinuteEntry>,
}

impl BenchmarkTable {
    /// `None` for a missing file, one that does not read, or another schema's.
    #[must_use]
    pub fn read(path: &Path) -> Option<Self> {
        Self::parse(&std::fs::read(path).ok()?)
    }

    pub(crate) fn parse(body: &[u8]) -> Option<Self> {
        let table: Self = serde_json::from_slice(body).ok()?;
        (table.schema_version == SCHEMA_VERSION).then_some(table)
    }

    /// False makes this a Stale table for the game on `patch`.
    #[must_use]
    pub fn serves(&self, patch: Patch) -> bool {
        if self.max_patch.is_some_and(|max| patch > max) {
            return false;
        }
        self.patch_window.contains(&patch)
            || self
                .patch_window
                .iter()
                .max()
                .is_some_and(|newest| newest.is_followed_by(patch))
    }

    pub(crate) fn take_items(&mut self) -> ItemTable {
        std::mem::take(&mut self.items)
    }

    /// The build already pooled a thin Cell's adjacent tiers into its entry, so no entry
    /// means no source. A neighbour's entry would be a different pool: see ADR 0004.
    #[must_use]
    pub fn into_cell(self, champion: &str, role: Role, tier: Tier) -> Option<CellBenchmarks> {
        let champion = champion_key(champion);
        self.cells
            .into_iter()
            .find(|cell| {
                champion_key(&cell.champion) == champion
                    && Role::parse(&cell.role) == Some(role)
                    && cell.tier == tier.table_label()
            })
            .map(|cell| CellBenchmarks::new(cell.minutes))
    }
}
