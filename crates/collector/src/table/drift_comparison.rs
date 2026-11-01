use lollipop_core::game_inputs::Patch;

/// The two patches of a Patch window.
#[derive(Clone, Copy)]
pub(super) struct DriftComparison {
    pub(super) older: Patch,
    pub(super) newer: Patch,
}

impl DriftComparison {
    pub(super) fn of_window(patches: &[Patch]) -> Option<Self> {
        let [.., older, newer] = patches else {
            return None;
        };
        Some(Self {
            older: *older,
            newer: *newer,
        })
    }
}
