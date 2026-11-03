//! Aggregates the stored matches into the Benchmark table file.

mod aligned_minute;
mod archived_game;
mod archived_timeline;
mod average;
mod benchmark_file;
mod candidate_list;
mod cell_drift;
mod cell_samples;
mod damage_profile;
mod data_dragon;
mod drift_comparison;
mod drift_report;
mod first_purchase;
mod item_candidates;
mod item_class;
mod match_items;
mod minute_samples;
mod participant_items;
mod patch_medians;
mod played_game;
mod profile_bucket;
mod reworks;
mod stored_payloads;
mod table_config;
mod table_error;

use std::path::Path;

use lollipop_core::game_inputs::Patch;

use crate::seed_tier::SeedTier;
use crate::store::Store;
use benchmark_file::{BenchmarkFile, CellEntry};
use cell_drift::CellDrift;
use cell_samples::CellSamples;
use damage_profile::DamageProfile;
use data_dragon::item_classes;
use drift_comparison::DriftComparison;
use item_candidates::item_candidates;
use match_items::MatchItems;

pub(crate) use benchmark_file::SCHEMA_VERSION;
pub use drift_report::DriftReport;
pub(crate) use played_game::PlayedGame;
pub(crate) use reworks::Reworks;
pub(crate) use stored_payloads::StoredPayloads;
pub use table_config::{TableConfig, DATA_DRAGON_URL, MIN_CANDIDATE_GAMES};
pub use table_error::TableError;

/// Writes the table and answers the drift report.
///
/// # Errors
/// On anything that stops a whole table being written. Any earlier file then stays.
pub fn build_table(config: &TableConfig) -> Result<DriftReport, TableError> {
    let reworks = Reworks::read(&config.reworks)?;
    let store = Store::open_read_only(&config.database)?;
    let patches = store.patch_window()?;
    let Some(&floor) = patches.first() else {
        return Err(TableError::NoMatches);
    };
    let comparison = DriftComparison::of_window(&patches);
    let mut build = Build {
        comparison,
        cells: Vec::new(),
        drifts: Vec::new(),
        open: None,
    };
    let classes = item_classes(&config.data_dragon_url, &patches)?;
    store.for_each_played_game(|game: PlayedGame| {
        if game.patch < floor || reworks.drops(&game.champion, game.patch) {
            return Ok::<(), TableError>(());
        }
        build.add(&game)
    })?;
    build.close_open_group();
    let mut matches = Vec::new();
    store.for_each_stored_payloads(floor, |stored| {
        matches.push(MatchItems::read(&stored, &classes)?);
        Ok::<(), TableError>(())
    })?;
    let profile = DamageProfile::of(&matches, &reworks);
    let candidates = profile.as_ref().map_or_else(Vec::new, |profile| {
        item_candidates(&matches, profile, &reworks, config.min_candidate_games)
    });
    let file = BenchmarkFile::new(
        config.max_patch,
        &patches,
        build.cells,
        profile.map(DamageProfile::entry),
        candidates,
    );
    write_atomically(&config.output, &file)?;
    Ok(DriftReport::new(&patches, comparison, &build.drifts))
}

struct Build {
    comparison: Option<DriftComparison>,
    cells: Vec<CellEntry>,
    drifts: Vec<CellDrift>,
    open: Option<CellSamples>,
}

impl Build {
    fn add(&mut self, game: &PlayedGame) -> Result<(), TableError> {
        let tier = SeedTier::from_label(&game.tier)
            .ok_or_else(|| TableError::UnknownTier(game.tier.clone()))?;
        let open = match self.open.take() {
            Some(open) if open.holds(&game.champion, &game.role) => open,
            finished => {
                self.close(finished);
                CellSamples::new(&game.champion, &game.role)
            }
        };
        self.open.insert(open).add(tier, game.patch, &game.frames);
        Ok(())
    }

    fn close_open_group(&mut self) {
        let open = self.open.take();
        self.close(open);
    }

    fn close(&mut self, group: Option<CellSamples>) {
        let Some(group) = group else {
            return;
        };
        self.cells.extend(group.cell_entries());
        if let Some(comparison) = self.comparison {
            self.drifts.extend(group.drifts(comparison));
        }
    }
}

// Through a rename, so a crash mid-write cannot leave a half-written table to publish.
fn write_atomically(output: &Path, file: &BenchmarkFile) -> Result<(), TableError> {
    let partial = output.with_extension("partial");
    let body = serde_json::to_vec(file).map_err(TableError::Serialize)?;
    std::fs::write(&partial, body).map_err(|error| TableError::Write(output.into(), error))?;
    std::fs::rename(&partial, output).map_err(|error| TableError::Write(output.into(), error))
}

pub(crate) fn patch_label(patch: Patch) -> String {
    format!("{}.{}", patch.major, patch.minor)
}
