use super::marker::Marker;

const HOLD_SECONDS: f64 = 30.0;

/// One metric's shown Marker, which follows the measured one only once it has held.
pub(super) struct HeldMarker {
    shown: Marker,
    pending: Option<Pending>,
}

struct Pending {
    marker: Marker,
    since: f64,
}

impl Default for HeldMarker {
    fn default() -> Self {
        Self {
            shown: Marker::Neutral,
            pending: None,
        }
    }
}

impl HeldMarker {
    /// `None` hides the Marker at once, and it comes back neutral: a state from before
    /// the gap says nothing about the Benchmark after it.
    pub(super) fn update(&mut self, game_time: f64, measured: Option<Marker>) -> Option<Marker> {
        let Some(measured) = measured else {
            *self = Self::default();
            return None;
        };
        if measured == self.shown {
            self.pending = None;
            return Some(self.shown);
        }
        let since = self
            .pending
            .as_ref()
            .filter(|pending| pending.marker == measured)
            .map_or(game_time, |pending| pending.since);
        if game_time - since >= HOLD_SECONDS {
            self.shown = measured;
            self.pending = None;
        } else {
            self.pending = Some(Pending {
                marker: measured,
                since,
            });
        }
        Some(self.shown)
    }
}
