use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TimingHint {
    pub objective: &'static str,
    /// Time to the spawn as `m:ss`, rounded up so the last second shown is `0:01`.
    pub countdown: String,
}
