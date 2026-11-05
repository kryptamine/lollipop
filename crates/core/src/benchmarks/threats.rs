use std::collections::HashMap;

use serde::Deserialize;

use super::champion_key::champion_key;

/// What enemy teams heal and mitigate, as the collector measured it.
#[derive(Debug, Deserialize)]
pub(super) struct Threats {
    pub(super) healing: ThreatProfile,
    pub(super) mitigation: ThreatProfile,
}

/// Per champion amount a minute, and the lowest team sum of the heavy quarter.
#[derive(Debug, Deserialize)]
pub(super) struct ThreatProfile {
    per_minute: HashMap<String, f64>,
    heavy_from: f64,
}

impl ThreatProfile {
    /// A champion with no amount counts 0, as the collector counts it.
    pub(super) fn heavy<'a>(&self, champions: impl Iterator<Item = &'a str>) -> bool {
        let sum: f64 = champions
            .filter_map(|champion| {
                let key = champion_key(champion);
                self.per_minute
                    .iter()
                    .find(|(name, _)| champion_key(name) == key)
                    .map(|(_, amount)| *amount)
            })
            .sum();
        sum >= self.heavy_from
    }
}
