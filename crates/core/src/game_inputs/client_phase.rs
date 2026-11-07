use std::path::Path;

use super::connection::Connection;
use super::lockfile::Lockfile;

const PHASE: &str = "/lol-gameflow/v1/gameflow-phase";

/// Where the League client is in the gameflow, which outlives Live Client Data: that
/// drops about 1 s after `GameEnd`, often before a poll sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClientPhase {
    /// `WaitingForStats`: the game is over and its stats are on their way.
    GameOver,
    /// `PreEndOfGame` or `EndOfGame`: the player is on the post-game screens.
    PostGame,
    /// Any other phase, such as `InProgress` or `Reconnect`.
    Other,
    /// No client answered.
    Unreadable,
}

impl ClientPhase {
    pub(crate) async fn read(http: &reqwest::Client, lockfile: &Path) -> Self {
        let Some(lockfile) = Lockfile::read(lockfile) else {
            return Self::Unreadable;
        };
        let phase: Option<String> = Connection::new(http, lockfile).get(PHASE).await;
        match phase.as_deref() {
            Some("WaitingForStats") => Self::GameOver,
            Some("PreEndOfGame" | "EndOfGame") => Self::PostGame,
            Some(_) => Self::Other,
            None => Self::Unreadable,
        }
    }

    pub(crate) const fn game_over(self) -> bool {
        matches!(self, Self::GameOver | Self::PostGame)
    }
}
