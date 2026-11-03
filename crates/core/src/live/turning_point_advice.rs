use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use super::advice_slot::AdviceSlot;
use super::answered::Answered;
use super::arrival::Arrival;
use super::fighting::fighting;
use super::payload::{AllGameData, Player};
use super::turning_point::{TurningPoint, TurningPointWatch};
use super::turning_point_prompt::{input, Turn, INSTRUCTIONS};
use crate::advice::{Advisor, Question, Trigger};

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
    asking: Option<Asking>,
}

struct Asking {
    game_time: f64,
    asked_at: f64,
    alive: bool,
    answer: JoinHandle<Option<Answered>>,
}

impl TurningPointAdvice {
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        player: &Player,
        advisor: &Advisor,
        slot: &mut AdviceSlot,
    ) {
        // A plan made while the player was alive no longer fits once they die.
        if player.is_dead && self.asking.as_ref().is_some_and(|asking| asking.alive) {
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
                    earlier: slot.shown(),
                };
                self.ask(&turn, advisor);
            }
        }
        self.collect(data, slot).await;
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.answer.abort();
        }
    }

    fn spaced(&self, game_time: f64) -> bool {
        self.shown_at.is_none_or(|shown| {
            let since = game_time - shown;
            since >= SPACING || since <= REPLACES_WITHIN
        })
    }

    fn ask(&mut self, turn: &Turn<'_>, advisor: &Advisor) {
        self.end_game();
        let Some(call) = advisor.call_for(Trigger::TurningPoint) else {
            return;
        };
        let input = input(call.language, turn);
        let TurningPoint { game_time, .. } = *turn.point;
        let deadline = Instant::now() + Duration::from_secs_f64(DROP_AFTER);
        let advisor = advisor.clone();
        self.asking = Some(Asking {
            game_time,
            asked_at: turn.data.game_time(),
            alive: !turn.player.is_dead,
            answer: tokio::spawn(async move {
                let question = Question {
                    trigger: Trigger::TurningPoint,
                    instructions: INSTRUCTIONS,
                    input: &input,
                    // No item is a candidate here, so naming one drops the whole Advice.
                    candidates: &[],
                    deadline,
                };
                Answered::of(advisor.ask(&call, &question).await)
            }),
        });
    }

    async fn collect(&mut self, data: &AllGameData, slot: &mut AdviceSlot) {
        let Some(asking) = self.asking.take_if(|asking| asking.answer.is_finished()) else {
            return;
        };
        if data.game_time() - asking.asked_at > DROP_AFTER {
            return;
        }
        let Some(answered) = asking.answer.await.ok().flatten() else {
            return;
        };
        let arrival = Arrival {
            trigger: Trigger::TurningPoint,
            text: answered.text,
            icon: None,
        };
        if slot.offer(arrival, Some(data.game_time())) {
            self.shown_at = Some(asking.game_time);
        }
    }
}
