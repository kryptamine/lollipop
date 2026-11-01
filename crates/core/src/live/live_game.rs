use std::sync::Arc;

use super::markers::Markers;
use super::snapshot::Snapshot;
use crate::game_inputs::GameInputs;

/// One poll of a running match: stats that move, and inputs that hold all game.
#[derive(Debug)]
pub struct LiveGame {
    pub snapshot: Snapshot,
    pub inputs: Arc<GameInputs>,
    pub markers: Markers,
}
