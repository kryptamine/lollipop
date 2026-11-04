use std::sync::Arc;
use std::time::Instant;

use super::advice_slot::AdviceSlot;
use super::arrival::Arrival;
use super::build::Build;
use super::fired::{AskInput, Fired};
use super::payload::AllGameData;
use super::pending_answer::PendingAnswer;
use super::proposals::Proposals;
use super::until::Until;
use crate::advice::{Advisor, AnswerParts, Question, Trigger};

/// One Trigger's Advice from asking to showing. A new ask replaces the one in flight,
/// and an answer past its [`Until`] is dropped with every Build proposal in it.
pub(super) struct AdviceAsk {
    trigger: Trigger,
    proposals: Proposals,
    instructions: Arc<str>,
    advisor: Advisor,
    asking: Option<(Until, PendingAnswer<AnswerParts>)>,
}

impl AdviceAsk {
    /// `own` is the Trigger's part of the instructions, without its Proposals'.
    pub(super) fn new(trigger: Trigger, own: &str, advisor: &Advisor) -> Self {
        let proposals = Proposals::of(trigger);
        Self {
            trigger,
            proposals,
            instructions: proposals.instructions(own).into(),
            advisor: advisor.clone(),
            asking: None,
        }
    }

    pub(super) fn fire(
        &mut self,
        fired: Fired<impl AskInput>,
        data: &AllGameData,
        build: &mut Build,
    ) {
        match fired {
            Fired::Ask { until, input } => self.ask(data, build, until, input),
            Fired::Drop => self.end(),
            Fired::Skip(why) => {
                self.end();
                self.log(why);
            }
        }
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
            self.log("dropped, late");
            return false;
        }
        let Some(text) = self.proposals.apply(parts, player, game_time, build) else {
            return false;
        };
        let arrival = Arrival {
            trigger: self.trigger,
            text,
        };
        let shown = slot.offer(arrival, game_time);
        self.log(if shown { "shown" } else { "dropped, outranked" });
        shown
    }

    pub(super) fn end(&mut self) {
        if self.asking.take().is_some() {
            self.log("dropped before its answer");
        }
    }

    /// Drops the answer on its way without a log line, as for a game that never ended.
    pub(super) fn forget(&mut self) {
        self.asking = None;
    }

    fn ask(&mut self, data: &AllGameData, build: &mut Build, until: Until, input: impl AskInput) {
        self.end();
        let Some(player) = data.active_player() else {
            return;
        };
        let Some(call) = self.advisor.call_for(self.trigger) else {
            self.log("off or no key, not asked");
            return;
        };
        if self.proposals.change_next_item() {
            build.track_lead(player);
        }
        let input = input(call.language, build);
        let options = self.proposals.lines(build, player, call.language);
        let deadline = Instant::now() + until.left(data, player);
        let (trigger, instructions) = (self.trigger, Arc::clone(&self.instructions));
        let advisor = self.advisor.clone();
        let answer = PendingAnswer::spawn(async move {
            let input = input.await + &options;
            let question = Question {
                trigger,
                instructions: &instructions,
                input: &input,
                deadline,
            };
            advisor.ask(&call, &question).await.into_parts()
        });
        self.asking = Some((until, answer));
    }

    /// One line naming this Trigger, as the Advisor logs its own.
    fn log(&self, what: &str) {
        self.advisor
            .log(&format!("Advice {}: {what}", self.trigger.id()));
    }
}
