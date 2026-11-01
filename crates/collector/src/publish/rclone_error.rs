use std::fmt;

/// An `rclone` run that failed, or that could not start.
#[derive(Debug)]
pub struct RcloneError {
    pub(super) command: String,
    pub(super) reason: String,
}

impl fmt::Display for RcloneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "rclone {}: {}", self.command, self.reason)
    }
}

impl std::error::Error for RcloneError {}
