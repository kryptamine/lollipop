use std::sync::Arc;

use crate::game_inputs::GameInputs;

/// Holds the current game's inputs, so that nothing read at its start can change.
#[derive(Default)]
pub(super) struct InputsOfGame {
    latest: Option<Latest>,
}

struct Latest {
    game_time: f64,
    inputs: Arc<GameInputs>,
}

impl InputsOfGame {
    /// A failed poll must not end the game: reading again would break "once per game".
    pub(super) fn end_game(&mut self) {
        self.latest = None;
    }

    /// `None` when this game has not been read yet.
    pub(super) fn held_at(&mut self, game_time: f64) -> Option<Arc<GameInputs>> {
        // Only a new game turns the clock back, however the last one ended.
        let latest = self.latest.as_mut().filter(|l| l.game_time <= game_time)?;
        latest.game_time = game_time;
        Some(Arc::clone(&latest.inputs))
    }

    pub(super) fn hold(&mut self, game_time: f64, inputs: GameInputs) -> Arc<GameInputs> {
        let inputs = Arc::new(inputs);
        self.latest = Some(Latest {
            game_time,
            inputs: Arc::clone(&inputs),
        });
        inputs
    }
}
