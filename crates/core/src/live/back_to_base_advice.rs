use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use super::advised_items::AdvisedItems;
use super::answered::Answered;
use super::back_to_base::BackToBaseWatch;
use super::back_to_base_prompt::{input, Visit, INSTRUCTIONS};
use super::item_widget::ItemWidget;
use super::payload::{AllGameData, Player};
use crate::advice::{Advisor, Question, Trigger};

// Seconds after the purchase from which a late answer is dropped.
const DROP_AFTER: f64 = 10.0;

/// Advice on what to buy, asked on the first purchase of each Back to base.
#[derive(Default)]
pub(super) struct BackToBaseAdvice {
    watch: BackToBaseWatch,
    asking: Option<Asking>,
}

struct Asking {
    bought_at: f64,
    answer: JoinHandle<Option<Answered>>,
}

impl BackToBaseAdvice {
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        player: &Player,
        advisor: &Advisor,
        widget: &mut ItemWidget<'_>,
    ) {
        // A death drops the answer: the shop it would advise on has closed.
        if player.is_dead {
            self.end_game();
        }
        if let Some(back) = self.watch.seen(data, player) {
            let visit = Visit {
                data,
                player,
                back: &back,
            };
            self.ask(&visit, advisor, widget.items);
        }
        self.collect(data, widget).await;
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.answer.abort();
        }
    }

    fn ask(&mut self, visit: &Visit<'_>, advisor: &Advisor, items: &AdvisedItems) {
        self.end_game();
        let Some(call) = advisor.call_for(Trigger::BackToBase) else {
            return;
        };
        let Visit { data, player, back } = visit;
        let candidates = items.candidates(player, data.enemies_of(player));
        let recommended = items.recommended(player);
        let input = input(call.language, visit, &candidates, &recommended);
        let ids: Vec<u32> = candidates.iter().map(|candidate| candidate.item).collect();
        let deadline = Instant::now() + Duration::from_secs_f64(DROP_AFTER);
        let advisor = advisor.clone();
        self.asking = Some(Asking {
            bought_at: back.game_time,
            answer: tokio::spawn(async move {
                let question = Question {
                    trigger: Trigger::BackToBase,
                    instructions: INSTRUCTIONS,
                    input: &input,
                    candidates: &ids,
                    deadline,
                };
                Answered::of(advisor.ask(&call, &question).await)
            }),
        });
    }

    async fn collect(&mut self, data: &AllGameData, widget: &mut ItemWidget<'_>) {
        let Some(asking) = self.asking.take_if(|asking| asking.answer.is_finished()) else {
            return;
        };
        if data.game_time() - asking.bought_at > DROP_AFTER {
            return;
        }
        if let Some(answered) = asking.answer.await.ok().flatten() {
            widget.offer(answered, Trigger::BackToBase, data.game_time());
        }
    }
}
