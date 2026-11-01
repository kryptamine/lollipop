use std::fmt;
use std::path::PathBuf;

use super::gh_error::GhError;
use super::rclone_error::RcloneError;
use crate::table::TableError;

#[derive(Debug)]
pub enum PublishError {
    Build(TableError),
    Refused(String),
    /// What the bucket holds cannot serve as the old table. A rollback repairs it.
    Baseline(String),
    NotKept(u32),
    Bucket(RcloneError),
    Issue(GhError),
    Write(PathBuf, std::io::Error),
    SigningKey(PathBuf, String),
    NoticeFailed(Box<Self>, GhError),
}

impl PublishError {
    pub(super) const fn issue_title(&self) -> &'static str {
        match self {
            Self::Build(_) => "Benchmark table build failed",
            Self::Refused(_) => "Benchmark table publish refused",
            _ => "Benchmark table publish failed",
        }
    }
}

impl fmt::Display for PublishError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(error) => write!(formatter, "build failed: {error}"),
            Self::Refused(reason) => write!(formatter, "publish refused: {reason}"),
            Self::Baseline(reason) => write!(
                formatter,
                "{reason}; point the manifest at a kept table with the rollback command"
            ),
            Self::NotKept(version) => write!(formatter, "table {version} is not kept"),
            Self::Bucket(error) => write!(formatter, "{error}"),
            Self::Issue(error) => write!(formatter, "{error}"),
            Self::Write(path, error) => {
                write!(formatter, "cannot write {}: {error}", path.display())
            }
            Self::SigningKey(path, reason) => {
                write!(formatter, "cannot use {}: {reason}", path.display())
            }
            Self::NoticeFailed(cause, notice) => {
                write!(
                    formatter,
                    "{cause}; the issue did not open either: {notice}"
                )
            }
        }
    }
}

impl std::error::Error for PublishError {}

impl From<TableError> for PublishError {
    fn from(error: TableError) -> Self {
        Self::Build(error)
    }
}

impl From<RcloneError> for PublishError {
    fn from(error: RcloneError) -> Self {
        Self::Bucket(error)
    }
}

impl From<GhError> for PublishError {
    fn from(error: GhError) -> Self {
        Self::Issue(error)
    }
}
