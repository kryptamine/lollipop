use std::fmt;

use lollipop_core::game_inputs::Patch;

use crate::table::patch_label;

/// An objective killed before the Objective schedule lets it spawn.
#[derive(Debug)]
pub(crate) struct EarlyKill {
    pub(super) objective: &'static str,
    pub(super) patch: Patch,
    pub(super) at_s: f64,
    pub(super) spawn_s: f64,
}

impl EarlyKill {
    /// Holds nothing that moves between builds, as it is how a repeat is recognised.
    pub(crate) fn issue_title(&self) -> String {
        format!(
            "{} killed before its scheduled spawn on patch {}",
            self.objective,
            patch_label(self.patch)
        )
    }

    pub(crate) fn issue_body(&self) -> String {
        format!(
            "The earliest stored {} kill on patch {} is at {}, and the Objective schedule has \
             no spawn before {}. Check the patch notes, and if the timer moved, add an entry \
             to `crates/core/src/live/objective_schedule.rs`.",
            self.objective,
            patch_label(self.patch),
            clock(self.at_s),
            clock(self.spawn_s)
        )
    }
}

impl fmt::Display for EarlyKill {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} on {} at {}, before {}",
            self.objective,
            patch_label(self.patch),
            clock(self.at_s),
            clock(self.spawn_s)
        )
    }
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "game times are small and never negative"
)]
fn clock(seconds: f64) -> String {
    let whole = seconds as u32;
    format!("{}:{:02}", whole / 60, whole % 60)
}
