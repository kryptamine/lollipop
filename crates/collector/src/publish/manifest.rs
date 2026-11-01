use std::path::Path;

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::bucket_assets::MANIFEST;
use super::manifest_signer::ManifestSigner;
use super::publish_error::PublishError;
use super::table_bucket::TableBucket;

/// The truth about which table is live: the app follows it, even to an older version.
/// A manifest with no signature still reads: it only needs signing.
#[derive(Deserialize)]
pub(super) struct Manifest {
    pub(super) version: u32,
    sha256: String,
    signature: Option<String>,
}

impl Manifest {
    pub(super) fn of(version: u32, table: &Path) -> Result<Self, PublishError> {
        let body =
            std::fs::read(table).map_err(|error| PublishError::Write(table.into(), error))?;
        Ok(Self {
            version,
            sha256: format!("{:x}", Sha256::digest(body)),
            signature: None,
        })
    }

    pub(super) fn read(path: &Path) -> Option<Self> {
        serde_json::from_slice(&std::fs::read(path).ok()?).ok()
    }

    pub(super) fn same_table(&self, other: &Self) -> bool {
        self.sha256 == other.sha256
    }

    // Ed25519 signatures are deterministic, so signing again gives the same bytes.
    pub(super) fn signed_by(&self, signer: &ManifestSigner) -> bool {
        self.signature.as_deref() == Some(signer.sign(self.version, &self.sha256).as_str())
    }

    pub(super) fn upload(
        &self,
        bucket: &TableBucket,
        signer: &ManifestSigner,
        work_dir: &Path,
    ) -> Result<(), PublishError> {
        let path = work_dir.join(MANIFEST);
        let body = json!({
            "version": self.version,
            "sha256": self.sha256,
            "signature": signer.sign(self.version, &self.sha256),
        });
        std::fs::write(&path, body.to_string())
            .map_err(|error| PublishError::Write(path.clone(), error))?;
        bucket.upload_manifest(&path, MANIFEST)?;
        Ok(())
    }
}
