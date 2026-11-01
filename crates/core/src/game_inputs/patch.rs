use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

/// Numbered as `gameVersion` numbers it: patch notes call 16.21 "26.21".
/// Ordered by `major`, then `minor`, so 17.1 follows 16.24.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Patch {
    pub major: u32,
    pub minor: u32,
}

impl Patch {
    /// Reads `16.21.8342754+branch.releases-16-21...`.
    #[must_use]
    pub fn parse(game_version: &str) -> Option<Self> {
        let mut numbers = game_version.split('.');
        Some(Self {
            major: numbers.next()?.parse().ok()?,
            minor: numbers.next()?.parse().ok()?,
        })
    }

    /// A Season's last patch number is not known in advance, so 17.1 follows any 16.x.
    #[must_use]
    pub const fn is_followed_by(self, next: Self) -> bool {
        (next.major == self.major && next.minor == self.minor + 1)
            || (next.major == self.major + 1 && next.minor == 1)
    }
}

impl<'de> Deserialize<'de> for Patch {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let label = String::deserialize(deserializer)?;
        Self::parse(&label).ok_or_else(|| D::Error::custom(format!("not a patch: {label}")))
    }
}
