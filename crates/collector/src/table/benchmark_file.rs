//! The Benchmark table file, one struct per nested object.

use std::collections::BTreeMap;

use lollipop_core::game_inputs::Patch;
use serde::Serialize;

use super::patch_label;

// Also the `v2/` of the path the file is published on.
pub(crate) const SCHEMA_VERSION: u32 = 2;

#[derive(Serialize)]
pub(super) struct BenchmarkFile {
    schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_patch: Option<String>,
    patch_window: Vec<String>,
    cells: Vec<CellEntry>,
    /// Left out when no stored match is archived, and then no list is either.
    #[serde(skip_serializing_if = "Option::is_none")]
    damage_profile: Option<DamageProfileEntry>,
    item_candidates: Vec<ItemCandidatesEntry>,
}

impl BenchmarkFile {
    pub(super) fn new(
        max_patch: Option<Patch>,
        patches: &[Patch],
        cells: Vec<CellEntry>,
        damage_profile: Option<DamageProfileEntry>,
        item_candidates: Vec<ItemCandidatesEntry>,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            max_patch: max_patch.map(patch_label),
            patch_window: patches.iter().copied().map(patch_label).collect(),
            cells,
            damage_profile,
            item_candidates,
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
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub(super) scores: Option<ScoresEntry>,
}

/// Means over the games kept with scores, which `scores_n` counts.
#[derive(Serialize)]
pub(super) struct ScoresEntry {
    pub(super) kills: f64,
    pub(super) deaths: f64,
    pub(super) assists: f64,
    pub(super) scores_n: usize,
}

/// Per champion magic share and the bounds of the Damage profile buckets. A team's
/// profile is the sum of its five shares, and each bound is the lowest of its bucket.
#[derive(Serialize)]
pub(super) struct DamageProfileEntry {
    pub(super) magic_share: BTreeMap<String, f64>,
    pub(super) mixed_from: f64,
    pub(super) magic_heavy_from: f64,
}

/// Item candidates of one champion and role, by the enemy team's Damage profile bucket.
#[derive(Serialize)]
pub(super) struct ItemCandidatesEntry {
    pub(super) champion: String,
    pub(super) role: String,
    pub(super) physical_heavy: Vec<CandidateEntry>,
    pub(super) mixed: Vec<CandidateEntry>,
    pub(super) magic_heavy: Vec<CandidateEntry>,
}

/// `share` of the games that bought `item`, and the median `minute` of its first purchase.
#[derive(Serialize)]
pub(super) struct CandidateEntry {
    pub(super) item: u32,
    pub(super) share: f64,
    pub(super) minute: f64,
}
