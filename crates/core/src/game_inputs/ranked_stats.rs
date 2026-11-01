//! The `/lol-ranked/v1/current-ranked-stats` JSON document.

use serde::Deserialize;

use super::tier::Tier;
use super::tier_reading::TierReading;

pub(super) const PATH: &str = "/lol-ranked/v1/current-ranked-stats";
const SOLO_DUO: &str = "RANKED_SOLO_5x5";

#[derive(Deserialize)]
pub(super) struct RankedStats {
    queues: Vec<QueueEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueueEntry {
    queue_type: String,
    tier: Option<String>,
}

impl RankedStats {
    pub(super) fn solo_duo_tier(&self) -> TierReading {
        let tier = self
            .queues
            .iter()
            .find(|entry| entry.queue_type == SOLO_DUO)
            .and_then(|entry| entry.tier.as_deref());
        match tier {
            Some("" | "NONE" | "UNRANKED") => TierReading::Unranked,
            Some(name) => Tier::parse(name).map_or(TierReading::Unknown, TierReading::Ranked),
            None => TierReading::Unknown,
        }
    }
}
