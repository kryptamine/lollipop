use std::fmt;
use std::process::Command;

const KEYCHAIN_SERVICE: &str = "lollipop-benchmark-token";

/// The read-only token for the private Release. It only ever comes from outside the binary.
pub struct ReleaseToken(String);

impl ReleaseToken {
    /// `None` when the Keychain has no such item, which leaves the app with no downloads.
    /// `security` created the item, so a rebuild's new ad-hoc signature brings no prompt,
    /// which would take focus from League.
    #[must_use]
    pub fn from_keychain() -> Option<Self> {
        let output = Command::new("security")
            .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let token = String::from_utf8(output.stdout).ok()?;
        let token = token.trim();
        (!token.is_empty()).then(|| Self::new(token))
    }

    #[must_use]
    pub fn new(token: &str) -> Self {
        Self(token.to_string())
    }

    pub(super) fn secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ReleaseToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ReleaseToken(..)")
    }
}
