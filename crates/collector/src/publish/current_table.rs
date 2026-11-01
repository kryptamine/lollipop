use std::path::Path;

use super::bucket_assets::{table_name, BucketAssets, MANIFEST};
use super::manifest::Manifest;
use super::publish_error::PublishError;
use super::table_bucket::TableBucket;
use super::table_summary::TableSummary;

const CURRENT_TABLE: &str = "current-table.json";

/// The live table, which the publish gate compares a new one to.
pub(super) struct CurrentTable {
    pub(super) manifest: Manifest,
    pub(super) summary: TableSummary,
}

impl CurrentTable {
    /// `None` only for a bucket with no tables. A manifest lost beside kept tables must
    /// not read as a first publish, which has no gate.
    pub(super) fn fetch(
        bucket: &TableBucket,
        assets: &BucketAssets,
        work_dir: &Path,
    ) -> Result<Option<Self>, PublishError> {
        if !assets.has_manifest() {
            if assets.holds_tables() {
                let reason = "the bucket holds tables but no manifest".to_string();
                return Err(PublishError::Baseline(reason));
            }
            return Ok(None);
        }
        let manifest_path = work_dir.join(MANIFEST);
        bucket.download(MANIFEST, &manifest_path)?;
        let manifest = Manifest::read(&manifest_path)
            .ok_or_else(|| PublishError::Baseline("the manifest does not read".to_string()))?;
        if !assets.holds(manifest.version) {
            let reason = format!("the manifest names table {}, not kept", manifest.version);
            return Err(PublishError::Baseline(reason));
        }
        let table_path = work_dir.join(CURRENT_TABLE);
        bucket.download(&table_name(manifest.version), &table_path)?;
        let summary = TableSummary::read(&table_path).map_err(PublishError::Baseline)?;
        Ok(Some(Self { manifest, summary }))
    }
}
