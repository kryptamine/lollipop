use std::fmt;

/// Names the call rather than its URL, which can carry a PUUID.
#[derive(Debug)]
pub enum CollectError {
    InvalidKey,
    KeyRejected(u16),
    CallFailed(&'static str),
    Unreadable(&'static str),
    Archive(std::io::Error),
    Client(reqwest::Error),
    Database(rusqlite::Error),
}

impl CollectError {
    /// A match that keeps failing is given up on, so that it cannot stall the walk.
    pub(crate) const fn only_costs_the_match(&self) -> bool {
        matches!(
            self,
            Self::CallFailed(_) | Self::Unreadable(_) | Self::Archive(_)
        )
    }
}

impl fmt::Display for CollectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey => write!(formatter, "the Riot key is not a valid header value"),
            Self::KeyRejected(status) => {
                write!(
                    formatter,
                    "Riot rejected the key with {status}: expired or invalid"
                )
            }
            Self::CallFailed(call) => write!(formatter, "{call} kept failing"),
            Self::Unreadable(call) => write!(formatter, "{call} answered an unreadable body"),
            Self::Archive(error) => {
                write!(formatter, "cannot compress a match to archive: {error}")
            }
            Self::Client(error) => write!(formatter, "cannot build the HTTP client: {error}"),
            Self::Database(error) => write!(formatter, "database: {error}"),
        }
    }
}

impl std::error::Error for CollectError {}

impl From<rusqlite::Error> for CollectError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}
