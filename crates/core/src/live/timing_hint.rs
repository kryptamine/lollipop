use serde::Serialize;

use super::build_item::BuildItem;

#[derive(Debug, Serialize)]
pub struct TimingHint {
    pub objective: &'static str,
    /// Time to the spawn as `m:ss`, rounded up so the last second shown is `0:01`.
    pub countdown: String,
    /// The Elixir to buy for Baron or Elder, only with a full inventory and its gold.
    pub elixir: Option<BuildItem>,
}
