use std::sync::Arc;

use super::markers::Markers;
use super::notice::Notice;
use super::shown_advice::ShownAdvice;
use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use super::timing_hint::TimingHint;
use crate::game_inputs::GameInputs;

const NOTICES_END_AT: f64 = 10.0;

/// One poll of a running match: stats that move, and inputs that hold all game.
#[derive(Debug)]
pub struct LiveGame {
    pub snapshot: Snapshot,
    pub inputs: Arc<GameInputs>,
    pub markers: Markers,
    pub benchmarks: ShownBenchmarks,
    pub hints: Vec<TimingHint>,
    pub advice: Option<ShownAdvice>,
    pub(super) notices: Arc<[Notice]>,
    pub(super) game_time: f64,
}

impl LiveGame {
    /// The strip's lines right now, none after the game's first 10 s.
    #[must_use]
    pub fn shown_notices(&self) -> Vec<String> {
        if self.game_time >= NOTICES_END_AT {
            return Vec::new();
        }
        self.notices.iter().map(ToString::to_string).collect()
    }
}
