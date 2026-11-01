//! Puts a built table on the GitHub Release the app reads, behind the publish gate.

mod current_table;
mod gh_error;
mod gh_release;
mod manifest;
mod publish_config;
mod publish_error;
mod publish_gate;
mod published;
mod release_assets;
mod rollback;
mod rollback_config;
mod table_summary;

use std::path::{Path, PathBuf};

use lollipop_core::game_inputs::Patch;

use crate::store::Store;
use crate::table::{build_table, DriftReport, Reworks, TableConfig, TableError};
use current_table::CurrentTable;
use gh_release::GhRelease;
use manifest::Manifest;
use publish_gate::refusal;
use release_assets::{table_name, ReleaseAssets};
use table_summary::TableSummary;

pub use gh_error::GhError;
pub use publish_config::PublishConfig;
pub use publish_error::PublishError;
pub use published::Published;
pub use rollback::rollback;
pub use rollback_config::RollbackConfig;

const BUILT_TABLE: &str = "benchmark-table.json";

/// Builds and publishes a table. Whatever stops it also opens an issue.
///
/// # Errors
/// On a failed build, a refused publish, or any `gh` failure, even after the manifest moved.
pub fn publish(config: &PublishConfig) -> Result<Published, PublishError> {
    let release = GhRelease::new(&config.repo);
    attempt(config, &release).map_err(|error| {
        match release.open_issue(error.issue_title(), &error.to_string()) {
            Ok(()) => error,
            Err(notice) => PublishError::NoticeFailed(Box::new(error), notice),
        }
    })
}

fn attempt(config: &PublishConfig, release: &GhRelease) -> Result<Published, PublishError> {
    std::fs::create_dir_all(&config.work_dir)
        .map_err(|error| PublishError::Write(config.work_dir.clone(), error))?;
    let reworks = Reworks::read(&config.reworks)?;
    let assets = release.assets()?;
    let current = CurrentTable::fetch(release, assets.as_ref(), &config.work_dir)?;
    let table = built_table(config, current.as_ref())?;
    let summary = TableSummary::read(&table).map_err(PublishError::Refused)?;
    if let Some(reason) = current
        .as_ref()
        .and_then(|current| refusal(&current.summary, &summary, &reworks))
    {
        return Err(PublishError::Refused(reason));
    }

    let version = assets.as_ref().map_or(1, ReleaseAssets::next_version);
    let manifest = Manifest::of(version, &table)?;
    if let Some(current) = current.filter(|current| current.manifest.same_table(&manifest)) {
        return Ok(Published::Unchanged(current.manifest.version));
    }
    if assets.is_none() {
        release.create()?;
    }
    upload_table(release, &table, &config.work_dir, version)?;
    manifest.upload(release, &config.work_dir)?;
    if let Some(assets) = assets {
        for outdated in assets.outdated_after_adding_one() {
            release.delete_asset(&table_name(*outdated))?;
        }
    }
    Ok(Published::New(version))
}

// The table goes up before a manifest can name it.
fn upload_table(
    release: &GhRelease,
    table: &Path,
    work_dir: &Path,
    version: u32,
) -> Result<(), PublishError> {
    let named = work_dir.join(table_name(version));
    std::fs::copy(table, &named).map_err(|error| PublishError::Write(named.clone(), error))?;
    release.upload(&named)?;
    Ok(())
}

fn built_table(
    config: &PublishConfig,
    current: Option<&CurrentTable>,
) -> Result<PathBuf, PublishError> {
    if let Some(table) = &config.table {
        return Ok(table.clone());
    }
    let inherited = current.and_then(|current| current.summary.max_patch);
    let output = config.work_dir.join(BUILT_TABLE);
    let table = TableConfig {
        database: config.database.clone(),
        reworks: config.reworks.clone(),
        output: output.clone(),
        max_patch: config.max_patch.or(still_in_force(inherited, config)?),
    };
    print_report(&build_table(&table)?);
    Ok(output)
}

// The weekly build passes no `--max-patch`, and must not lift one until data passes it.
fn still_in_force(
    inherited: Option<Patch>,
    config: &PublishConfig,
) -> Result<Option<Patch>, TableError> {
    let window = Store::open_read_only(&config.database)?.patch_window()?;
    Ok(inherited.filter(|max_patch| window.last().is_some_and(|newest| newest <= max_patch)))
}

#[allow(clippy::print_stdout, reason = "each build prints its drift report")]
fn print_report(report: &DriftReport) {
    println!("{report}");
}
