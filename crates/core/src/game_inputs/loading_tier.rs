use std::path::Path;

use super::connection::Connection;
use super::lockfile::Lockfile;
use super::ranked_stats::{self, RankedStats};
use super::tier_reading::TierReading;

/// The player's solo/duo tier on the loading screen, before `GameInputs` are read.
pub(crate) async fn loading_tier(http: &reqwest::Client, lockfile: &Path) -> TierReading {
    let Some(lockfile) = Lockfile::read(lockfile) else {
        return TierReading::Unknown;
    };
    Connection::new(http, lockfile)
        .get::<RankedStats>(ranked_stats::PATH)
        .await
        .map_or(TierReading::Unknown, |ranked| ranked.solo_duo_tier())
}
