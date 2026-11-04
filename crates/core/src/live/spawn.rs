use super::clock_text::countdown;
use super::timing_hint::TimingHint;

const LEAD_SECONDS: f64 = 45.0;

/// The next time an objective appears, in seconds of game time.
pub(super) struct Spawn {
    pub(super) objective: &'static str,
    pub(super) at: f64,
}

impl Spawn {
    /// `None` outside the lead: too early, or the objective has already spawned.
    pub(super) fn hint_at(&self, game_time: f64) -> Option<TimingHint> {
        let left = self.at - game_time;
        (left > 0.0 && left <= LEAD_SECONDS).then(|| TimingHint {
            objective: self.objective,
            countdown: countdown(left),
            elixir: None,
        })
    }
}
