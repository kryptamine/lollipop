//! The `league-v4` JSON documents a seed player is read from.

use serde::Deserialize;

/// A PUUID stays in memory: `Store` has no column that takes one.
#[derive(Deserialize)]
pub(crate) struct LeagueEntry {
    pub(crate) puuid: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct ApexLeague {
    pub(crate) entries: Vec<LeagueEntry>,
}
