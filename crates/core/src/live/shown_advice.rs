use serde::Serialize;

use crate::advice::Trigger;

/// The Advice the Widget shows right now.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownAdvice {
    pub trigger: Trigger,
    pub text: String,
    /// The share of its time on screen still to go, from 1 down to 0.
    pub time_left: f64,
}
