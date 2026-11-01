use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::manifest_key::ManifestKey;
use super::signed_message::signed_message;

pub(super) const MANIFEST_OBJECT: &str = "manifest.json";

/// The truth about which table is live. Any version it names is followed, older included.
#[derive(Debug, Deserialize)]
pub(super) struct Manifest {
    version: u32,
    pub(super) sha256: String,
    signature: String,
}

impl Manifest {
    /// `None` for a body that does not read, or that this key did not sign.
    pub(super) fn parse(body: &[u8], key: &ManifestKey) -> Option<Self> {
        let manifest: Self = serde_json::from_slice(body).ok()?;
        let message = signed_message(manifest.version, &manifest.sha256);
        key.verifies(&message, &manifest.signature)
            .then_some(manifest)
    }

    pub(super) fn table_object(&self) -> String {
        format!("benchmark-table-{}.json", self.version)
    }

    pub(super) fn names(&self, table: &[u8]) -> bool {
        format!("{:x}", Sha256::digest(table)) == self.sha256
    }
}
