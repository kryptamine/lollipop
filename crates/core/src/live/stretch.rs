use serde::Serialize;

/// A span of game time, in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Stretch {
    pub from: f64,
    pub to: f64,
}
