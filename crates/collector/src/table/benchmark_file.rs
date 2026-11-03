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
    #[serde(flatten)]
    items: ItemsEntry,
}

impl BenchmarkFile {
    pub(super) fn new(
        max_patch: Option<Patch>,
        patches: &[Patch],
        cells: Vec<CellEntry>,
        items: ItemsEntry,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            max_patch: max_patch.map(patch_label),
            patch_window: patches.iter().copied().map(patch_label).collect(),
            cells,
            items,
        }
    }
}

/// What item Advice reads, at the top level of the file beside `cells`.
#[derive(Serialize)]
pub(super) struct ItemsEntry {
    /// Left out when no stored match is archived, and then no list is either.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) damage_profile: Option<DamageProfileEntry>,
    pub(super) item_candidates: Vec<ItemCandidatesEntry>,
    /// The Data Dragon version the items were read at, for their icons.
    pub(super) data_dragon_version: String,
    /// The en and ru name of every candidate id, for the prompt.
    pub(super) item_names: BTreeMap<u32, ItemNameEntry>,
    /// Every boots id past the first tier, candidate or not: a player holds one pair.
    pub(super) boots: Vec<u32>,
    /// Every completed item id, candidate or not, so the app sees a player finish one.
    pub(super) completed: Vec<u32>,
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
    /// The transforms that hold `item`, so a player owning one is not advised it again.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) owned_as: Vec<u32>,
    /// Every component down the item's Data Dragon `from` tree, which a player is building.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) builds_from: Vec<u32>,
}

#[derive(Clone, Serialize)]
pub(super) struct ItemNameEntry {
    pub(super) en: String,
    pub(super) ru: String,
}
