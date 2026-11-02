use serde::Serialize;

/// Kills, deaths and assists a Cell averages at one moment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ScoresAt {
    pub kills: f64,
    pub deaths: f64,
    pub assists: f64,
}

impl ScoresAt {
    /// Kills and assists less deaths: one number for a single Marker, which a ratio
    /// would not give with no deaths yet.
    #[must_use]
    pub fn net(&self) -> f64 {
        self.kills + self.assists - self.deaths
    }
}
