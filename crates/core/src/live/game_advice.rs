use std::path::Path;

use super::advice_ask::AdviceAsk;
use super::advice_slot::AdviceSlot;
use super::build::Build;
use super::death_advice::DeathAdvice;
use super::game_facts::GameFacts;
use super::loading_screen_advice::LoadingScreenAdvice;
use super::payload::AllGameData;
use super::shown_advice::ShownAdvice;
use super::turning_point_advice::TurningPointAdvice;
use super::{death_prompt, loading_screen_prompt, turning_point_prompt};
use crate::advice::{Advisor, Trigger};

/// Every Trigger of the game being played, the asks they fire, and the one Advice their
/// answers compete for. A Trigger only says when it fires; this owns the rest.
pub(super) struct GameAdvice {
    advisor: Advisor,
    loading_screen: LoadingScreenAdvice,
    death: DeathAdvice,
    turning_point: TurningPointAdvice,
    loading_screen_ask: AdviceAsk,
    death_ask: AdviceAsk,
    turning_point_ask: AdviceAsk,
    slot: AdviceSlot,
}

impl GameAdvice {
    pub(super) fn new(http: &reqwest::Client, lockfile: &Path, advisor: &Advisor) -> Self {
        Self {
            advisor: advisor.clone(),
            loading_screen: LoadingScreenAdvice::new(http, lockfile),
            death: DeathAdvice::default(),
            turning_point: TurningPointAdvice::default(),
            loading_screen_ask: ask(Trigger::LoadingScreen, advisor),
            death_ask: ask(Trigger::Death, advisor),
            turning_point_ask: ask(Trigger::TurningPoint, advisor),
            slot: AdviceSlot::default(),
        }
    }

    pub(super) async fn at_loading_screen(&mut self, data: &AllGameData, build: &mut Build) {
        if let Some(fired) = self.loading_screen.at_loading_screen(data) {
            // A new loading screen is a new game: the last one's Advice goes with it.
            self.slot.clear();
            self.loading_screen_ask.fire(fired, data, build);
        }
        self.loading_screen_ask
            .collect(data, None, &mut self.slot, build)
            .await;
    }

    /// Answers are collected in rank order, so a higher one arriving at the same poll wins.
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        facts: &GameFacts<'_>,
        build: &mut Build,
    ) {
        let game_time = Some(data.game_time());
        self.loading_screen.rearm();
        self.loading_screen_ask
            .collect(data, game_time, &mut self.slot, build)
            .await;

        if let Some(fired) = self.turning_point.at(data, facts, self.slot.shown()) {
            self.turning_point_ask.fire(fired, data, build);
        }
        let shown = self
            .turning_point_ask
            .collect(data, game_time, &mut self.slot, build)
            .await;
        if shown {
            self.turning_point.shown();
        }

        if let Some(fired) = self.death.at(data, facts) {
            self.death_ask.fire(fired, data, build);
        }
        self.death_ask
            .collect(data, game_time, &mut self.slot, build)
            .await;
    }

    /// Keeps the loading screen's answer, which may still be on its way.
    pub(super) fn start_game(&mut self) {
        self.death = DeathAdvice::default();
        self.turning_point = TurningPointAdvice::default();
        self.death_ask.forget();
        self.turning_point_ask.forget();
    }

    pub(super) fn end_game(&mut self) {
        self.loading_screen.rearm();
        self.turning_point.end_game();
        self.loading_screen_ask.end();
        self.death_ask.end();
        self.turning_point_ask.end();
        self.slot.clear();
        self.advisor.end_game();
    }

    pub(super) fn shown(&mut self, game_time: f64, game_start: Option<f64>) -> Option<ShownAdvice> {
        self.slot.at(game_time, game_start)
    }
}

fn ask(trigger: Trigger, advisor: &Advisor) -> AdviceAsk {
    let own = match trigger {
        Trigger::LoadingScreen => loading_screen_prompt::INSTRUCTIONS,
        Trigger::TurningPoint => turning_point_prompt::INSTRUCTIONS,
        Trigger::Death => death_prompt::INSTRUCTIONS,
    };
    AdviceAsk::new(trigger, own, advisor)
}
