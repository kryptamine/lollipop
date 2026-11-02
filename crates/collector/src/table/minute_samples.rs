use super::aligned_minute::AlignedMinute;

/// Every game's values at one minute, in one Cell on one patch or pooled across them.
#[derive(Default)]
pub(super) struct MinuteSamples {
    pub(super) cs: Vec<f64>,
    pub(super) total_gold: Vec<f64>,
    pub(super) level: Vec<f64>,
    /// Only from games kept with scores, so fewer than the others while old games remain.
    pub(super) kills: Vec<f64>,
    pub(super) deaths: Vec<f64>,
    pub(super) assists: Vec<f64>,
}

impl MinuteSamples {
    pub(super) fn add(&mut self, aligned: &AlignedMinute) {
        self.cs.push(aligned.cs);
        self.total_gold.push(aligned.total_gold);
        self.level.push(aligned.level);
        if let Some([kills, deaths, assists]) = aligned.scores {
            self.kills.push(kills);
            self.deaths.push(deaths);
            self.assists.push(assists);
        }
    }

    pub(super) fn extend(&mut self, other: &Self) {
        self.cs.extend_from_slice(&other.cs);
        self.total_gold.extend_from_slice(&other.total_gold);
        self.level.extend_from_slice(&other.level);
        self.kills.extend_from_slice(&other.kills);
        self.deaths.extend_from_slice(&other.deaths);
        self.assists.extend_from_slice(&other.assists);
    }

    pub(super) const fn n(&self) -> usize {
        self.cs.len()
    }
}
