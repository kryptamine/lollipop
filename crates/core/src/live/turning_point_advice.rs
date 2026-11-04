use std::future::ready;

use super::build::Build;
use super::fighting::fighting;
use super::fired::{AskInput, Fired, InputFuture};
use super::game_facts::GameFacts;
use super::payload::AllGameData;
use super::turning_point::{TurningPoint, TurningPointWatch};
use super::turning_point_prompt::{input, Turn};
use super::until::Until;
use crate::advice::Language;

// Seconds after the request from which a late answer is dropped.
const DROP_AFTER: f64 = 15.0;
// From the last Turning point shown, during which another one is skipped.
const SPACING: f64 = 180.0;
// Inside the spacing, a Turning point this close to the last one shown replaces it.
const REPLACES_WITHIN: f64 = 30.0;

/// A macro plan, asked when a moment changes the game plan.
#[derive(Default)]
pub(super) struct TurningPointAdvice {
    watch: TurningPointWatch,
    /// The `gameTime` of the last Turning point whose Advice showed. One dropped or
    /// answered null starts no window.
    shown_at: Option<f64>,
    /// A Turning point seen while the player fights, asked once the fight has passed.
    held: Option<TurningPoint>,
    /// The last Turning point asked about, kept once its answer is in.
    asked: Option<Asked>,
}

struct Asked {
    game_time: f64,
    alive: bool,
}

impl TurningPointAdvice {
    /// `earlier` holds the texts of the Advice this game showed, oldest first.
    pub(super) fn at<'a>(
        &mut self,
        data: &'a AllGameData,
        facts: &'a GameFacts<'a>,
        earlier: &'a [String],
    ) -> Option<Fired<impl AskInput + 'a>> {
        let player = data.active_player()?;
        // A plan made while the player was alive no longer fits once they die.
        let fired = if player.is_dead && self.asked.as_ref().is_some_and(|asked| asked.alive) {
            self.end_game();
            Some(Fired::Drop)
        } else {
            None
        };
        if let Some(point) = self.watch.seen(data) {
            if self.spaced(point.game_time) {
                self.held = Some(point);
            }
        }
        if fighting(data, player) {
            return fired;
        }
        let Some(point) = self.held.take() else {
            return fired;
        };
        self.asked = Some(Asked {
            game_time: point.game_time,
            alive: !player.is_dead,
        });
        Some(Fired::Ask {
            until: Until::GameTime(data.game_time() + DROP_AFTER),
            input: move |language: Language, _: &Build| -> InputFuture {
                let turn = Turn {
                    data,
                    player,
                    point: &point,
                    facts,
                    earlier,
                };
                Box::pin(ready(input(language, &turn)))
            },
        })
    }

    /// The last Turning point asked about starts the spacing window.
    pub(super) fn shown(&mut self) {
        self.shown_at = self.asked.as_ref().map(|asked| asked.game_time);
    }

    pub(super) const fn end_game(&mut self) {
        self.asked = None;
    }

    fn spaced(&self, game_time: f64) -> bool {
        self.shown_at.is_none_or(|shown| {
            let since = game_time - shown;
            since >= SPACING || since <= REPLACES_WITHIN
        })
    }
}
