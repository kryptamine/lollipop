use super::bucket_assets::table_name;
use super::manifest::Manifest;
use super::manifest_signer::ManifestSigner;
use super::publish_error::PublishError;
use super::rollback_config::RollbackConfig;
use super::table_bucket::TableBucket;
use super::table_summary::TableSummary;

/// # Errors
/// When the table is not kept or does not read, or `rclone` fails. Run by hand, so no issue
/// opens. It skips the gate, as a kept table has passed it once.
pub fn rollback(config: &RollbackConfig) -> Result<(), PublishError> {
    std::fs::create_dir_all(&config.work_dir)
        .map_err(|error| PublishError::Write(config.work_dir.clone(), error))?;
    let signer = ManifestSigner::read(&config.signing_key)?;
    let bucket = TableBucket::new(&config.bucket);
    if !bucket.assets()?.holds(config.version) {
        return Err(PublishError::NotKept(config.version));
    }
    let name = table_name(config.version);
    let table = config.work_dir.join(&name);
    bucket.download(&name, &table)?;
    TableSummary::read(&table).map_err(PublishError::Refused)?;
    Manifest::of(config.version, &table)?.upload(&bucket, &signer, &config.work_dir)
}
