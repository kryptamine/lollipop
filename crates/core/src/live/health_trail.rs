use std::collections::VecDeque;

use super::payload::AllGameData;

// Live Client Data has no death recap, so the last seconds of health are the only sign
// of a burst or a long trade.
const KEPT_FOR: f64 = 10.0;

/// The active player's health share over their last seconds alive, a poll apart.
#[derive(Default)]
pub(super) struct HealthTrail {
    /// Game time and health over max health, oldest first.
    samples: VecDeque<(f64, f64)>,
}

impl HealthTrail {
    /// A death clears it, so one life's health never reads as the next one's.
    pub(super) fn observe(&mut self, data: &AllGameData, dead: bool) {
        if dead {
            self.samples.clear();
            return;
        }
        let Some(share) = data.health_share() else {
            return;
        };
        let now = data.game_time();
        // `gameTime` stands still while the game is paused.
        if self.samples.back().is_some_and(|(at, _)| *at >= now) {
            self.samples.pop_back();
        }
        self.samples.push_back((now, share));
        while self
            .samples
            .front()
            .is_some_and(|(at, _)| now - at > KEPT_FOR)
        {
            self.samples.pop_front();
        }
    }

    /// The samples of the last seconds before `died_at`, oldest first.
    pub(super) fn before(&self, died_at: f64) -> impl Iterator<Item = (f64, f64)> + '_ {
        self.samples
            .iter()
            .copied()
            .filter(move |(at, _)| died_at - at <= KEPT_FOR)
    }
}
