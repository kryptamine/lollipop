use std::future::Future;
use std::time::Instant;

use super::advice_slot::AdviceSlot;
use super::arrival::Arrival;
use super::build::Build;
use super::payload::{AllGameData, Player};
use super::pending_answer::PendingAnswer;
use super::until::Until;
use crate::advice::{Advisor, AnswerParts, Language, Question, Trigger};

/// One Trigger's Advice from asking to showing. A new ask replaces the one in flight,
/// and an answer past its [`Until`] is dropped with every Build proposal in it.
pub(super) struct AdviceAsk {
    trigger: Trigger,
    instructions: &'static str,
    advisor: Advisor,
    asking: Option<(Until, PendingAnswer<AnswerParts>)>,
}

impl AdviceAsk {
    pub(super) fn new(trigger: Trigger, instructions: &'static str, advisor: &Advisor) -> Self {
        Self {
            trigger,
            instructions,
            advisor: advisor.clone(),
            asking: None,
        }
    }

    /// `input` reads the Build once its lead is tracked; its future runs with the call.
    pub(super) fn ask<Input>(
        &mut self,
        data: &AllGameData,
        build: &mut Build,
        until: Until,
        input: impl FnOnce(Language, &Build) -> Input,
    ) where
        Input: Future<Output = String> + Send + 'static,
    {
        self.end();
        let Some(player) = data.active_player() else {
            return;
        };
        let Some(call) = self.advisor.call_for(self.trigger) else {
            return;
        };
        if changes_next_item(self.trigger) {
            build.track_lead(player);
        }
        let input = input(call.language, build);
        let deadline = Instant::now() + until.left(data, player);
        let (trigger, instructions) = (self.trigger, self.instructions);
        let advisor = self.advisor.clone();
        let answer = PendingAnswer::spawn(async move {
            let input = input.await;
            let question = Question {
                trigger,
                instructions,
                input: &input,
                deadline,
            };
            advisor.ask(&call, &question).await.into_parts()
        });
        self.asking = Some((until, answer));
    }

    /// Whether the Advice Widget took the answer's text. `game_time` is `None` on the
    /// loading screen.
    pub(super) async fn collect(
        &mut self,
        data: &AllGameData,
        game_time: Option<f64>,
        slot: &mut AdviceSlot,
        build: &mut Build,
    ) -> bool {
        let Some((until, answer)) = &mut self.asking else {
            return false;
        };
        let until = *until;
        let Some(parts) = answer.finished().await else {
            return false;
        };
        self.asking = None;
        let player = data.active_player();
        if until.passed(game_time, player) {
            return false;
        }
        let Some(text) = self.apply(parts, player, game_time, build) else {
            return false;
        };
        let arrival = Arrival {
            trigger: self.trigger,
            text,
        };
        slot.offer(arrival, game_time)
    }

    pub(super) fn end(&mut self) {
        self.asking = None;
    }

    /// Hands the Build what this Trigger may change, and returns the text.
    fn apply(
        &self,
        parts: AnswerParts,
        player: Option<&Player>,
        game_time: Option<f64>,
        build: &mut Build,
    ) -> Option<String> {
        let AnswerParts {
            text,
            change,
            swap,
            elixir,
        } = parts;
        let Some(player) = player.filter(|_| changes_next_item(self.trigger)) else {
            return text;
        };
        if let Some(change) = change {
            build.change(change, player, game_time);
        }
        if self.trigger == Trigger::Death {
            if let Some(swap) = swap {
                build.swap(swap, player);
            }
            if let Some(elixir) = elixir {
                build.pick_elixir(elixir, player);
            }
        }
        text
    }
}

// ADR 0018 opens the Next item to the loading screen and a death alone, and only the
// death prompt offers a Swap or an Elixir.
const fn changes_next_item(trigger: Trigger) -> bool {
    matches!(trigger, Trigger::LoadingScreen | Trigger::Death)
}
