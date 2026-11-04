use std::future::ready;

use super::advice_ask::AdviceAsk;
use super::advice_slot::AdviceSlot;
use super::build::Build;
use super::fighting::fighting;
use super::game_facts::GameFacts;
use super::payload::AllGameData;
use super::turning_point::{TurningPoint, TurningPointWatch};
use super::turning_point_prompt::{input, Turn, INSTRUCTIONS};
use super::until::Until;
use crate::advice::{Advisor, Trigger};

// Seconds after the request from which a late answer is dropped.
const DROP_AFTER: f64 = 15.0;
// From the last Turning point shown, during which another one is skipped.
const SPACING: f64 = 180.0;
// Inside the spacing, a Turning point this close to the last one shown replaces it.
const REPLACES_WITHIN: f64 = 30.0;

/// A macro plan, asked when a moment changes the game plan.
pub(super) struct TurningPointAdvice {
    watch: TurningPointWatch,
    /// The `gameTime` of the last Turning point whose Advice showed. One dropped or
    /// answered null starts no window.
    shown_at: Option<f64>,
    /// A Turning point seen while the player fights, asked once the fight has passed.
    held: Option<TurningPoint>,
    /// The last Turning point asked about, kept once its answer is in.
    asked: Option<Asked>,
    ask: AdviceAsk,
}

struct Asked {
    game_time: f64,
    alive: bool,
}

impl TurningPointAdvice {
    pub(super) fn new(advisor: &Advisor) -> Self {
        Self {
            watch: TurningPointWatch::default(),
            shown_at: None,
            held: None,
            asked: None,
            ask: AdviceAsk::new(Trigger::TurningPoint, INSTRUCTIONS, advisor),
        }
    }

    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        facts: &GameFacts<'_>,
        slot: &mut AdviceSlot,
        build: &mut Build,
    ) {
        let Some(player) = data.active_player() else {
            return;
        };
        // A plan made while the player was alive no longer fits once they die.
        if player.is_dead && self.asked.as_ref().is_some_and(|asked| asked.alive) {
            self.end_game();
        }
        if let Some(point) = self.watch.seen(data) {
            if self.spaced(point.game_time) {
                self.held = Some(point);
            }
        }
        if !fighting(data, player) {
            if let Some(point) = self.held.take() {
                let turn = Turn {
                    data,
                    player,
                    point: &point,
                    facts,
                    earlier: slot.shown(),
                };
                self.ask(&turn, build);
            }
        }
        let shown = self
            .ask
            .collect(data, Some(data.game_time()), slot, build)
            .await;
        if shown {
            self.shown_at = self.asked.as_ref().map(|asked| asked.game_time);
        }
    }

    pub(super) fn end_game(&mut self) {
        self.asked = None;
        self.ask.end();
    }

    fn spaced(&self, game_time: f64) -> bool {
        self.shown_at.is_none_or(|shown| {
            let since = game_time - shown;
            since >= SPACING || since <= REPLACES_WITHIN
        })
    }

    fn ask(&mut self, turn: &Turn<'_>, build: &mut Build) {
        self.end_game();
        let Turn { data, player, .. } = *turn;
        let until = Until::GameTime(data.game_time() + DROP_AFTER);
        self.ask.ask(data, build, until, |language, _| {
            ready(input(language, turn))
        });
        self.asked = Some(Asked {
            game_time: turn.point.game_time,
            alive: !player.is_dead,
        });
    }
}
