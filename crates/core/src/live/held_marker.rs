use super::comparison::Comparison;
use super::marker::Marker;

// A triangle shows as soon as the value leaves the Neutral zone, and stays until the
// value is back within half of it, so a wave on the edge does not flicker it.
const KEEP_SHARE: f64 = 0.5;

/// One metric's shown Marker.
#[derive(Default)]
pub(super) struct HeldMarker {
    shown: Option<Marker>,
}

impl HeldMarker {
    /// `None` hides the Marker and forgets it: a state from before the gap says nothing
    /// about the Benchmark after it.
    pub(super) fn update(&mut self, measured: Option<Comparison>) -> Option<Marker> {
        let Some(measured) = measured else {
            self.shown = None;
            return None;
        };
        let kept = self
            .shown
            .filter(|&shown| shown != Marker::Neutral && measured.marker(KEEP_SHARE) == shown);
        self.shown = Some(kept.unwrap_or_else(|| measured.marker(1.0)));
        self.shown
    }
}
