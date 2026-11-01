use std::path::Path;
use std::process::Command;

use super::bucket_assets::BucketAssets;
use super::publish_error::PublishError;
use super::rclone_error::RcloneError;
use crate::table::SCHEMA_VERSION;

// Set in the unit's environment as RCLONE_CONFIG_TABLES_*.
const REMOTE: &str = "tables";
// A rollback must reach the apps soon, and the next check after a game is the only retry.
const MANIFEST_CACHE: &str = "Cache-Control: public, max-age=60";
// A version is never reused, so its file never changes.
const TABLE_CACHE: &str = "Cache-Control: public, max-age=31536000, immutable";

/// The public bucket of one schema: its `v1` prefix is the app's `v1/manifest.json` path.
pub(super) struct TableBucket {
    prefix: String,
}

impl TableBucket {
    pub(super) fn new(bucket: &str) -> Self {
        Self {
            prefix: format!("{REMOTE}:{bucket}/v{SCHEMA_VERSION}"),
        }
    }

    /// An empty listing is a first publish: storage has no empty prefix to tell apart.
    pub(super) fn assets(&self) -> Result<BucketAssets, RcloneError> {
        let listed = run(&["lsf", &self.prefix, "--files-only"])?;
        Ok(BucketAssets::new(&listed))
    }

    pub(super) fn download(&self, asset: &str, to: &Path) -> Result<(), PublishError> {
        // rclone exits 0 for a missing object, and a leftover would read as the download.
        if to.exists() {
            std::fs::remove_file(to).map_err(|error| PublishError::Write(to.into(), error))?;
        }
        run(&["copyto", &self.object(asset), &to.to_string_lossy()])?;
        Ok(())
    }

    pub(super) fn upload_table(&self, file: &Path, asset: &str) -> Result<(), RcloneError> {
        self.upload(file, asset, TABLE_CACHE)
    }

    pub(super) fn upload_manifest(&self, file: &Path, asset: &str) -> Result<(), RcloneError> {
        self.upload(file, asset, MANIFEST_CACHE)
    }

    pub(super) fn delete(&self, asset: &str) -> Result<(), RcloneError> {
        run(&["deletefile", &self.object(asset)]).map(drop)
    }

    fn upload(&self, file: &Path, asset: &str, cache: &str) -> Result<(), RcloneError> {
        let to = self.object(asset);
        let args = [
            "copyto",
            &file.to_string_lossy(),
            &to,
            "--header-upload",
            cache,
        ];
        run(&args).map(drop)
    }

    fn object(&self, asset: &str) -> String {
        format!("{}/{asset}", self.prefix)
    }
}

fn run(args: &[&str]) -> Result<String, RcloneError> {
    let command = args.first().copied().unwrap_or_default().to_string();
    let output = Command::new("rclone")
        .args(args)
        .output()
        .map_err(|error| RcloneError {
            command: command.clone(),
            reason: error.to_string(),
        })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(RcloneError {
        command,
        reason: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}
