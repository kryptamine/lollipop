use std::path::Path;

use ed25519_dalek::{Signer, SigningKey};
use lollipop_core::delivery::signed_message;

use super::publish_error::PublishError;

/// The secret half of the key compiled into the app, kept apart from the updater's.
pub(super) struct ManifestSigner(SigningKey);

impl ManifestSigner {
    /// The file holds the 32-byte seed in hex. An error never quotes it.
    pub(super) fn read(path: &Path) -> Result<Self, PublishError> {
        let unreadable = |reason: String| PublishError::SigningKey(path.into(), reason);
        let seed = std::fs::read_to_string(path).map_err(|error| unreadable(error.to_string()))?;
        let mut bytes = [0; 32];
        hex::decode_to_slice(seed.trim(), &mut bytes)
            .map_err(|_| unreadable("it is not 32 bytes of hex".to_string()))?;
        Ok(Self(SigningKey::from_bytes(&bytes)))
    }

    pub(super) fn sign(&self, version: u32, sha256: &str) -> String {
        let signature = self.0.sign(signed_message(version, sha256).as_bytes());
        hex::encode(signature.to_bytes())
    }

    pub(super) fn public_key(&self) -> String {
        hex::encode(self.0.verifying_key().as_bytes())
    }
}

/// The hex the app compiles in for this signing key.
///
/// # Errors
/// When the file does not read as a signing key.
pub fn manifest_public_key(path: &Path) -> Result<String, PublishError> {
    Ok(ManifestSigner::read(path)?.public_key())
}
