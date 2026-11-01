//! The Benchmark table file, one struct per nested object.

use lollipop_core::game_inputs::Patch;
use serde::Serialize;

use super::patch_label;

// Also the `v1/` of the path the file is published on.
const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
pub(super) struct BenchmarkFile {
    schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_patch: Option<String>,
    patch_window: Vec<String>,
    cells: Vec<CellEntry>,
}

impl BenchmarkFile {
    pub(super) fn new(max_patch: Option<Patch>, patches: &[Patch], cells: Vec<CellEntry>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            max_patch: max_patch.map(patch_label),
            patch_window: patches.iter().copied().map(patch_label).collect(),
            cells,
        }
    }
}

#[derive(Serialize)]
pub(super) struct CellEntry {
    pub(super) champion: String,
    pub(super) role: String,
    pub(super) tier: &'static str,
    pub(super) minutes: Vec<MinuteEntry>,
}

#[derive(Serialize)]
pub(super) struct MinuteEntry {
    pub(super) minute: usize,
    pub(super) cs: f64,
    pub(super) total_gold: f64,
    pub(super) level: f64,
    pub(super) n: usize,
}
