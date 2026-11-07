use serde::Serialize;

/// The player at the first poll of one game minute, against the Cell at that minute.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecapMinute {
    pub minute: u32,
    pub cs: u32,
    pub gold_earned: Option<u32>,
    /// `None` wherever the Marker strip shows no Benchmark, as for a support's CS.
    pub cs_benchmark: Option<f64>,
    pub gold_benchmark: Option<f64>,
}
