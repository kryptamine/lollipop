use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Marker {
    Above,
    Below,
    Neutral,
}

impl Marker {
    /// Exactly on the edge of the Neutral zone is still inside it.
    pub(super) fn of(value: f64, benchmark: f64, neutral_zone: f64) -> Self {
        let distance = value - benchmark;
        if distance > neutral_zone {
            Self::Above
        } else if distance < -neutral_zone {
            Self::Below
        } else {
            Self::Neutral
        }
    }
}
