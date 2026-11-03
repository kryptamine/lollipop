use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum TableError {
    Reworks(PathBuf, String),
    NoMatches,
    UnknownTier(String),
    DataDragon(String),
    /// A match's archived payloads, by its id, do not read.
    Archive(String, String),
    Database(rusqlite::Error),
    Serialize(serde_json::Error),
    Write(PathBuf, std::io::Error),
}

impl fmt::Display for TableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reworks(path, reason) => {
                write!(formatter, "cannot use {}: {reason}", path.display())
            }
            Self::NoMatches => write!(formatter, "the database holds no matches"),
            Self::UnknownTier(tier) => write!(formatter, "a stored match has tier {tier}"),
            Self::DataDragon(reason) => write!(formatter, "Data Dragon: {reason}"),
            Self::Archive(match_id, reason) => {
                write!(
                    formatter,
                    "the archive of {match_id} does not read: {reason}"
                )
            }
            Self::Database(error) => write!(formatter, "database: {error}"),
            Self::Serialize(error) => write!(formatter, "cannot serialize the table: {error}"),
            Self::Write(path, error) => {
                write!(formatter, "cannot write {}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for TableError {}

impl From<rusqlite::Error> for TableError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}
