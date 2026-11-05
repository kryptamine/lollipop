use std::collections::VecDeque;

use super::cs_pace::CsPace;
use super::cs_step::CsStep;
use super::likely_cs::likely_cs;
use super::per_minute::per_minute;

const WINDOW: f64 = 180.0;

/// One game's CS steps. The window starts at the latest step at least three minutes back,
/// so a stretch with no step stretches it: a start inside one would guess up to 9 CS.
#[derive(Default)]
pub(super) struct CsPaceWatch {
    last_cs: Option<u32>,
    steps: VecDeque<CsStep>,
}

impl CsPaceWatch {
    /// `None` until a step lies three minutes back: the first poll of a game is no step.
    pub(super) fn observe(&mut self, game_time: f64, cs: u32) -> Option<CsPace> {
        if self.last_cs.replace(cs).is_some_and(|last| cs > last) {
            self.steps.push_back(CsStep { at: game_time, cs });
        }
        let window_starts = game_time - WINDOW;
        while self
            .steps
            .get(1)
            .is_some_and(|step| step.at <= window_starts)
        {
            self.steps.pop_front();
        }
        let start = self.steps.front().filter(|step| step.at <= window_starts)?;
        let gained = likely_cs(cs) - f64::from(start.cs);
        Some(CsPace {
            since: start.at,
            per_minute: per_minute(gained, start.at, game_time),
        })
    }
}
