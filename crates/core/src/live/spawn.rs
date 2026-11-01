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
        })
    }
}

fn countdown(left: f64) -> String {
    let seconds = whole_seconds(left);
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "no std conversion from f64; the time left is positive and at most the lead"
)]
const fn whole_seconds(left: f64) -> u32 {
    left.ceil() as u32
}
