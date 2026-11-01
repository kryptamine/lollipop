use super::aligned_minute::AlignedMinute;

/// Every game's values at one minute, in one Cell on one patch or pooled across them.
#[derive(Default)]
pub(super) struct MinuteSamples {
    pub(super) cs: Vec<f64>,
    pub(super) total_gold: Vec<f64>,
    pub(super) level: Vec<f64>,
}

impl MinuteSamples {
    pub(super) fn add(&mut self, aligned: &AlignedMinute) {
        self.cs.push(aligned.cs);
        self.total_gold.push(aligned.total_gold);
        self.level.push(aligned.level);
    }

    pub(super) fn extend(&mut self, other: &Self) {
        self.cs.extend_from_slice(&other.cs);
        self.total_gold.extend_from_slice(&other.total_gold);
        self.level.extend_from_slice(&other.level);
    }

    pub(super) const fn n(&self) -> usize {
        self.cs.len()
    }
}
