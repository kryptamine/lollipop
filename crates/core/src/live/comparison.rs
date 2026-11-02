use super::marker::Marker;

/// One metric's compared value against its Benchmark at one poll.
#[derive(Debug, Clone, Copy)]
pub(super) struct Comparison {
    pub(super) value: f64,
    pub(super) benchmark: f64,
    pub(super) neutral_zone: f64,
}

impl Comparison {
    /// `share` scales the Neutral zone: a whole one to enter a state, a part to keep it.
    pub(super) fn marker(self, share: f64) -> Marker {
        Marker::of(self.value, self.benchmark, self.neutral_zone * share)
    }
}
