use serde::Serialize;

use super::scores::Scores;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub cs: u32,
    pub level: u32,
    /// `None` for a game first seen too late to count, which the view shows as a dash.
    pub gold_earned: Option<u32>,
    /// `None` when the payload lacks them, which the view shows as no K/D/A row.
    pub scores: Option<Scores>,
    /// CS a minute over at least the last three minutes, `None` until three minutes of CS
    /// are seen, which the view shows as a dash.
    pub cs_pace: Option<f64>,
}
