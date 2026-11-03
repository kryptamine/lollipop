//! Puts a built table in the public bucket the app reads, behind the publish gate.

mod bucket_assets;
mod current_table;
mod gh_error;
mod gh_issues;
mod manifest;
mod manifest_signer;
mod publish_config;
mod publish_error;
mod publish_gate;
mod published;
mod rclone_error;
mod rollback;
mod rollback_config;
mod table_bucket;
mod table_summary;

use std::fmt::Display;
use std::path::{Path, PathBuf};

use lollipop_core::game_inputs::Patch;

use crate::schedule_check::check_schedule;
use crate::store::Store;
use crate::table::{build_table, Reworks, TableConfig, TableError};
use bucket_assets::table_name;
use current_table::CurrentTable;
use gh_issues::GhIssues;
use manifest::Manifest;
use manifest_signer::ManifestSigner;
use publish_gate::refusal;
use table_bucket::TableBucket;
use table_summary::TableSummary;

pub use gh_error::GhError;
pub use manifest_signer::manifest_public_key;
pub use publish_config::PublishConfig;
pub use publish_error::PublishError;
pub use published::Published;
pub use rclone_error::RcloneError;
pub use rollback::rollback;
pub use rollback_config::RollbackConfig;

const BUILT_TABLE: &str = "benchmark-table.json";

/// # Errors
/// On a failed build, a refused publish, an unreadable signing key, or any `rclone` or `gh`
/// failure, even after the manifest moved.
/// Whatever stops it also opens an issue.
pub fn publish(config: &PublishConfig) -> Result<Published, PublishError> {
    let issues = GhIssues::new(&config.repo);
    attempt(config, &issues).map_err(|error| {
        match issues.open(error.issue_title(), &error.to_string()) {
            Ok(()) => error,
            Err(notice) => PublishError::NoticeFailed(Box::new(error), notice),
        }
    })
}

fn attempt(config: &PublishConfig, issues: &GhIssues) -> Result<Published, PublishError> {
    std::fs::create_dir_all(&config.work_dir)
        .map_err(|error| PublishError::Write(config.work_dir.clone(), error))?;
    let signer = ManifestSigner::read(&config.signing_key)?;
    let reworks = Reworks::read(&config.reworks)?;
    let bucket = TableBucket::new(&config.bucket);
    let assets = bucket.assets()?;
    let current = CurrentTable::fetch(&bucket, &assets, &config.work_dir)?;
    let table = built_table(config, issues, current.as_ref())?;
    let summary = TableSummary::read(&table).map_err(PublishError::Refused)?;
    if let Some(reason) = current
        .as_ref()
        .and_then(|current| refusal(&current.summary, &summary, &reworks))
    {
        return Err(PublishError::Refused(reason));
    }

    let version = assets.next_version();
    let manifest = Manifest::of(version, &table)?;
    if let Some(current) = current.filter(|current| current.manifest.same_table(&manifest)) {
        if !current.manifest.signed_by(&signer) {
            current
                .manifest
                .upload(&bucket, &signer, &config.work_dir)?;
        }
        return Ok(Published::Unchanged(current.manifest.version));
    }
    upload_table(&bucket, &table, &config.work_dir, version)?;
    manifest.upload(&bucket, &signer, &config.work_dir)?;
    for outdated in assets.outdated_after_adding_one() {
        bucket.delete(&table_name(*outdated))?;
    }
    Ok(Published::New(version))
}

// The table goes up before a manifest can name it.
fn upload_table(
    bucket: &TableBucket,
    table: &Path,
    work_dir: &Path,
    version: u32,
) -> Result<(), PublishError> {
    let named = work_dir.join(table_name(version));
    std::fs::copy(table, &named).map_err(|error| PublishError::Write(named.clone(), error))?;
    bucket.upload_table(&named, &table_name(version))?;
    Ok(())
}

fn built_table(
    config: &PublishConfig,
    issues: &GhIssues,
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
        data_dragon_url: config.data_dragon_url.clone(),
        min_candidate_games: config.min_candidate_games,
    };
    print_report(&build_table(&table)?);
    report_schedule(config, issues)?;
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

fn report_schedule(config: &PublishConfig, issues: &GhIssues) -> Result<(), PublishError> {
    let check = check_schedule(&config.database)?;
    print_report(&check);
    for kill in check.early_kills() {
        issues.open_once(&kill.issue_title(), &kill.issue_body())?;
    }
    Ok(())
}

#[allow(clippy::print_stdout, reason = "each build prints what it found")]
fn print_report(report: &impl Display) {
    println!("{report}");
}
