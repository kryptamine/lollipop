use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::gh_release::GhRelease;
use super::publish_error::PublishError;
use super::release_assets::MANIFEST;

/// The truth about which table is live: the app follows it, even to an older version.
#[derive(Serialize, Deserialize)]
pub(super) struct Manifest {
    pub(super) version: u32,
    sha256: String,
}

impl Manifest {
    pub(super) fn of(version: u32, table: &Path) -> Result<Self, PublishError> {
        let body =
            std::fs::read(table).map_err(|error| PublishError::Write(table.into(), error))?;
        Ok(Self {
            version,
            sha256: format!("{:x}", Sha256::digest(body)),
        })
    }

    pub(super) fn read(path: &Path) -> Option<Self> {
        serde_json::from_slice(&std::fs::read(path).ok()?).ok()
    }

    pub(super) fn same_table(&self, other: &Self) -> bool {
        self.sha256 == other.sha256
    }

    pub(super) fn upload(&self, release: &GhRelease, work_dir: &Path) -> Result<(), PublishError> {
        let path = work_dir.join(MANIFEST);
        let body = serde_json::to_vec(self)
            .map_err(|error| PublishError::Write(path.clone(), error.into()))?;
        std::fs::write(&path, body).map_err(|error| PublishError::Write(path.clone(), error))?;
        release.upload(&path)?;
        Ok(())
    }
}
