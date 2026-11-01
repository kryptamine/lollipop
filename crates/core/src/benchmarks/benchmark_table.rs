use std::path::Path;

use serde::Deserialize;

use super::cell_benchmarks::{CellBenchmarks, MinuteEntry};
use super::champion_key::champion_key;
use crate::game_inputs::{Role, Tier};

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
pub struct BenchmarkTable {
    schema_version: u32,
    cells: Vec<CellEntry>,
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
        let body = std::fs::read(path).ok()?;
        let table: Self = serde_json::from_slice(&body).ok()?;
        (table.schema_version == SCHEMA_VERSION).then_some(table)
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
