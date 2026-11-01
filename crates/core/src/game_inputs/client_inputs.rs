use super::client_reading::ClientReading;
use super::connection::Connection;
use super::gameflow_session;
use super::patch::Patch;
use super::ranked_stats::{self, RankedStats};
use super::tier_reading::TierReading;

/// Later Widgets tell these apart: an unavailable client also stops Timing hints.
#[derive(Debug, PartialEq, Eq)]
pub enum ClientInputs {
    Unavailable,
    Read(ClientReading),
}

impl ClientInputs {
    pub(super) async fn read(connection: &Connection<'_>) -> Self {
        let (queue, version, ranked) = tokio::join!(
            gameflow_session::queue_id(connection),
            connection.get::<String>("/lol-patch/v1/game-version"),
            connection.get::<RankedStats>(ranked_stats::PATH),
        );
        if queue.is_none() && version.is_none() && ranked.is_none() {
            return Self::Unavailable;
        }
        Self::Read(ClientReading {
            queue,
            patch: version.as_deref().and_then(Patch::parse),
            tier: ranked.map_or(TierReading::Unknown, |ranked| ranked.solo_duo_tier()),
        })
    }
}
