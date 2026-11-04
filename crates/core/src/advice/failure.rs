use std::fmt;
use std::time::Duration;

use super::key_failure::KeyFailure;

/// Why one Advice call came to nothing, named as the log names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Failure {
    Network,
    Timeout,
    /// A 429 rate limit, with the wait its `Retry-After` asked for.
    RateLimited {
        retry_after: Option<Duration>,
    },
    /// A 429 `insufficient_quota`: the key works, but its project has no credit.
    NoQuota,
    Status(u16),
    Schema,
    Length,
}

impl Failure {
    /// The failures a retry may mend, which the Widget reports when the retry fails too.
    pub(super) const fn is_unreachable(self) -> bool {
        match self {
            Self::Network | Self::Timeout | Self::RateLimited { .. } => true,
            Self::Status(code) => code >= 500,
            Self::NoQuota | Self::Schema | Self::Length => false,
        }
    }

    pub(super) const fn key_failure(self) -> Option<KeyFailure> {
        match self {
            Self::Status(401) => Some(KeyFailure::Rejected),
            Self::NoQuota => Some(KeyFailure::NoQuota),
            _ => None,
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network => f.write_str("network"),
            Self::Timeout => f.write_str("timeout"),
            Self::RateLimited { .. } => f.write_str("http 429"),
            Self::NoQuota => f.write_str("http 429 insufficient_quota"),
            Self::Status(code) => write!(f, "http {code}"),
            Self::Schema => f.write_str("schema"),
            Self::Length => f.write_str("length"),
        }
    }
}
