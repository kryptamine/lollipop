use std::collections::HashMap;

use serde::Deserialize;

use super::champion_key::champion_key;

/// Per champion magic share, and the lowest team sum of the mixed and magic-heavy buckets.
#[derive(Debug, Deserialize)]
pub(super) struct DamageProfile {
    magic_share: HashMap<String, f64>,
    mixed_from: f64,
    magic_heavy_from: f64,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum ProfileBucket {
    PhysicalHeavy,
    Mixed,
    MagicHeavy,
}

impl DamageProfile {
    /// A champion the file has no share for counts as all physical, as the collector counts it.
    pub(super) fn bucket_of<'a>(&self, team: impl Iterator<Item = &'a str>) -> ProfileBucket {
        let sum: f64 = team.filter_map(|champion| self.share_of(champion)).sum();
        if sum >= self.magic_heavy_from {
            ProfileBucket::MagicHeavy
        } else if sum >= self.mixed_from {
            ProfileBucket::Mixed
        } else {
            ProfileBucket::PhysicalHeavy
        }
    }

    pub(super) fn share_of(&self, champion: &str) -> Option<f64> {
        let key = champion_key(champion);
        self.magic_share
            .iter()
            .find(|(name, _)| champion_key(name) == key)
            .map(|(_, share)| *share)
    }
}
