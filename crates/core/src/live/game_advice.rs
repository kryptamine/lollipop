use std::path::Path;

use super::advice_slot::AdviceSlot;
use super::build::Build;
use super::death_advice::DeathAdvice;
use super::loading_screen_advice::LoadingScreenAdvice;
use super::payload::{AllGameData, Player};
use super::shown_advice::ShownAdvice;
use super::turning_point_advice::TurningPointAdvice;
use crate::advice::Advisor;

/// Every Trigger of the game being played, and the one Advice their answers compete for.
pub(super) struct GameAdvice {
    advisor: Advisor,
    loading_screen: LoadingScreenAdvice,
    death: DeathAdvice,
    turning_point: TurningPointAdvice,
    slot: AdviceSlot,
}

impl GameAdvice {
    pub(super) fn new(http: &reqwest::Client, lockfile: &Path, advisor: &Advisor) -> Self {
        Self {
            advisor: advisor.clone(),
            loading_screen: LoadingScreenAdvice::new(http, lockfile, advisor),
            death: DeathAdvice::new(advisor),
            turning_point: TurningPointAdvice::new(advisor),
            slot: AdviceSlot::default(),
        }
    }

    pub(super) async fn at_loading_screen(&mut self, data: &AllGameData, build: &mut Build) {
        self.loading_screen
            .at_loading_screen(data, &mut self.slot, build)
            .await;
    }

    pub(super) async fn at(&mut self, data: &AllGameData, player: &Player, build: &mut Build) {
        self.loading_screen.at(data, &mut self.slot, build).await;
        self.turning_point
            .at(data, player, &mut self.slot, build)
            .await;
        self.death.at(data, &mut self.slot, build).await;
    }

    /// Keeps the loading screen's answer, which may still be on its way.
    pub(super) fn start_game(&mut self) {
        self.death = DeathAdvice::new(&self.advisor);
        self.turning_point = TurningPointAdvice::new(&self.advisor);
    }

    pub(super) fn end_game(&mut self) {
        self.loading_screen.end_game();
        self.death.end_game();
        self.turning_point.end_game();
        self.slot.clear();
        self.advisor.end_game();
    }

    pub(super) fn shown(&mut self, game_time: f64, game_start: Option<f64>) -> Option<ShownAdvice> {
        self.slot.at(game_time, game_start)
    }
}
