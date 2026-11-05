use serde::Serialize;

use crate::benchmarks::ScoresAt;

/// The Benchmarks the strip shows beside its Markers, `None` wherever a Marker is.
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownBenchmarks {
    pub cs: Option<f64>,
    pub gold: Option<f64>,
    pub level: Option<f64>,
    pub scores: Option<ScoresAt>,
    /// What the Cell gains a minute over the same stretch as the player's CS pace.
    pub cs_pace: Option<f64>,
}
