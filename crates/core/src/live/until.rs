use std::time::Duration;

use super::payload::{AllGameData, Player};

/// How long a Trigger's answer is worth showing: both the Advisor's deadline and the
/// check that drops a late answer.
#[derive(Clone, Copy)]
pub(super) enum Until {
    Respawn,
    GameTime(f64),
}

impl Until {
    pub(super) fn left(self, data: &AllGameData, player: &Player) -> Duration {
        let seconds = match self {
            Self::Respawn => player.respawn_timer,
            Self::GameTime(at) => at - data.game_time(),
        };
        Duration::from_secs_f64(seconds.max(0.0))
    }

    /// `game_time` is `None` on the loading screen, where no answer is late yet.
    pub(super) fn passed(self, game_time: Option<f64>, player: Option<&Player>) -> bool {
        match self {
            Self::Respawn => !player.is_some_and(|player| player.is_dead),
            Self::GameTime(at) => game_time.is_some_and(|now| now >= at),
        }
    }
}
