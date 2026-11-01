use serde::Deserialize;
use sha2::{Digest, Sha256};

/// The truth about which table is live. Any version it names is followed, older included.
#[derive(Debug, Deserialize)]
pub(super) struct Manifest {
    pub(super) version: u32,
    pub(super) sha256: String,
}

impl Manifest {
    pub(super) fn parse(body: &[u8]) -> Option<Self> {
        serde_json::from_slice(body).ok()
    }

    pub(super) fn names(&self, table: &[u8]) -> bool {
        format!("{:x}", Sha256::digest(table)) == self.sha256
    }
}
