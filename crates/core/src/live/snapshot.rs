use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub cs: u32,
    pub level: u32,
    /// `None` for a game first seen too late to count, which the view shows as a dash.
    pub gold_earned: Option<u32>,
}
