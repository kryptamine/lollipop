use ed25519_dalek::{Signature, VerifyingKey};

// Not the updater's key. Its secret half is on the collector's host.
const COMPILED_IN: &str = "1f796920b979aa7e15ca608db8364e390d214f06607e506d6b34a55cef116cec";

/// The public half of the key the collector signs manifests with.
#[derive(Debug)]
pub struct ManifestKey(VerifyingKey);

impl ManifestKey {
    /// The key of the tables the app actually follows.
    #[must_use]
    pub fn compiled_in() -> Option<Self> {
        Self::from_hex(COMPILED_IN)
    }

    /// `None` for anything that is not an encoded public key.
    #[must_use]
    pub fn from_hex(key: &str) -> Option<Self> {
        let mut bytes = [0; 32];
        hex::decode_to_slice(key, &mut bytes).ok()?;
        VerifyingKey::from_bytes(&bytes).ok().map(Self)
    }

    pub(super) fn verifies(&self, message: &str, signature: &str) -> bool {
        let mut bytes = [0; 64];
        hex::decode_to_slice(signature, &mut bytes).is_ok()
            && self
                .0
                .verify_strict(message.as_bytes(), &Signature::from_bytes(&bytes))
                .is_ok()
    }
}
