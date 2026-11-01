use std::fmt;

/// A `gh` run that failed, or that could not start.
#[derive(Debug)]
pub struct GhError {
    pub(super) command: String,
    pub(super) reason: String,
}

impl fmt::Display for GhError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "gh {}: {}", self.command, self.reason)
    }
}

impl std::error::Error for GhError {}
