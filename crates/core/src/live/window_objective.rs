use serde::Serialize;

/// An objective to take while one side has more champions alive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowObjective {
    pub objective: &'static str,
    /// As `m:ss`; `None` while it is up.
    pub spawns_in: Option<String>,
}
