use super::frame::Frame;

/// Games of one participant, with frames exactly on the minute and steady growth.
pub struct Played {
    pub champion: &'static str,
    pub role: &'static str,
    pub tier: &'static str,
    pub patch: (u32, u32),
    pub minutes: u32,
    pub cs_per_minute: u32,
    pub gold_per_minute: u32,
    pub level: u32,
    /// Kills, deaths and assists gained each minute, `None` for a game with no scores.
    pub scores_per_minute: Option<[u32; 3]>,
}

impl Played {
    pub const fn new(champion: &'static str, tier: &'static str) -> Self {
        Self {
            champion,
            role: "TOP",
            tier,
            patch: (16, 21),
            minutes: 25,
            cs_per_minute: 7,
            gold_per_minute: 400,
            level: 10,
            scores_per_minute: None,
        }
    }

    pub fn frames(&self) -> Vec<Frame> {
        (0..=self.minutes)
            .map(|minute| Frame {
                at_ms: u64::from(minute) * 60_000,
                cs: self.cs_per_minute * minute,
                total_gold: self.gold_per_minute * minute,
                level: self.level,
                scores: self
                    .scores_per_minute
                    .map(|rates| rates.map(|rate| rate * minute)),
            })
            .collect()
    }
}
