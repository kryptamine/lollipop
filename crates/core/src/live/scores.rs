use serde::Serialize;

/// The active player's kills, deaths and assists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Scores {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

impl Scores {
    pub(super) fn net(self) -> f64 {
        f64::from(self.kills) + f64::from(self.assists) - f64::from(self.deaths)
    }
}
