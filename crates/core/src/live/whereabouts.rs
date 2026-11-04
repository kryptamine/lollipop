use serde::Serialize;

use super::place::Place;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Whereabouts {
    /// The respawn countdown as `m:ss`.
    #[serde(rename_all = "camelCase")]
    Dead { back_in: String },
    /// The time since as `m:ss`.
    Seen { place: Place, ago: String },
}
