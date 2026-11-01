use super::gh_release::GhRelease;
use super::manifest::Manifest;
use super::publish_error::PublishError;
use super::release_assets::table_name;
use super::rollback_config::RollbackConfig;
use super::table_summary::TableSummary;

/// # Errors
/// When the table is not kept or does not read, or `gh` fails. Run by hand, so no issue opens.
/// It skips the gate, as a kept table has passed it once.
pub fn rollback(config: &RollbackConfig) -> Result<(), PublishError> {
    std::fs::create_dir_all(&config.work_dir)
        .map_err(|error| PublishError::Write(config.work_dir.clone(), error))?;
    let release = GhRelease::new(&config.repo);
    let kept = release
        .assets()?
        .is_some_and(|assets| assets.holds(config.version));
    if !kept {
        return Err(PublishError::NotKept(config.version));
    }
    let name = table_name(config.version);
    let table = config.work_dir.join(&name);
    release.download(&name, &table)?;
    TableSummary::read(&table).map_err(PublishError::Refused)?;
    Manifest::of(config.version, &table)?.upload(&release, &config.work_dir)
}
