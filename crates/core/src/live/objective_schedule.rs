use crate::game_inputs::Patch;

const fn minutes(minutes: f64) -> f64 {
    minutes * 60.0
}

/// When objectives appear, in seconds of game time. See ADR 0005.
pub(super) struct ObjectiveSchedule {
    from_patch: Patch,
    pub(super) drake_first: f64,
    pub(super) drake_respawn: f64,
    pub(super) elder_after_fourth_drake: f64,
    pub(super) elder_respawn: f64,
    pub(super) voidgrubs_first: f64,
    pub(super) herald_first: f64,
    pub(super) baron_first: f64,
    pub(super) baron_respawn: f64,
}

// Oldest first. An entry may name a patch not yet out, so a fix can ship ahead of it.
// A Season's first entry starts at its first patch, as hints run on any patch within it.
static SCHEDULES: [ObjectiveSchedule; 1] = [ObjectiveSchedule {
    from_patch: Patch {
        major: 16,
        minor: 1,
    },
    drake_first: minutes(5.0),
    drake_respawn: minutes(5.0),
    elder_after_fourth_drake: minutes(6.0),
    elder_respawn: minutes(6.0),
    voidgrubs_first: minutes(8.0),
    herald_first: minutes(15.0),
    baron_first: minutes(20.0),
    baron_respawn: minutes(6.0),
}];

impl ObjectiveSchedule {
    /// `None` on a Season with no entry: last Season's timers may well be wrong for it.
    pub(super) fn for_patch(patch: Patch) -> Option<&'static Self> {
        SCHEDULES
            .iter()
            .rev()
            .find(|entry| entry.from_patch.major == patch.major && entry.from_patch <= patch)
    }
}
