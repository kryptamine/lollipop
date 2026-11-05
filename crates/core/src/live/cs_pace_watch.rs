use std::collections::VecDeque;

use super::compared_game::CS_OFFSET;
use super::recent_cs::RecentCs;

const WINDOW: f64 = 180.0;
const SECONDS_PER_MINUTE: f64 = 60.0;

/// One game's CS steps: the polls at which the floored CS went up, the only ones at which
/// the count is exact. The window starts at the latest step at least three minutes back.
#[derive(Default)]
pub(super) struct CsPaceWatch {
    last_cs: Option<u32>,
    steps: VecDeque<(f64, u32)>,
}

impl CsPaceWatch {
    /// `None` until a step lies three minutes back: the first poll of a game is no step.
    pub(super) fn observe(&mut self, game_time: f64, cs: u32) -> Option<RecentCs> {
        if self.last_cs.replace(cs).is_some_and(|last| cs > last) {
            self.steps.push_back((game_time, cs));
        }
        let window_starts = game_time - WINDOW;
        while self
            .steps
            .get(1)
            .is_some_and(|&(at, _)| at <= window_starts)
        {
            self.steps.pop_front();
        }
        let &(since, cs_then) = self.steps.front().filter(|&&(at, _)| at <= window_starts)?;
        let gained = f64::from(cs) + CS_OFFSET - f64::from(cs_then);
        Some(RecentCs {
            since,
            per_minute: gained / ((game_time - since) / SECONDS_PER_MINUTE),
        })
    }
}
