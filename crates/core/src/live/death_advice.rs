use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use super::advised_items::AdvisedItems;
use super::answered::Answered;
use super::death::Death;
use super::death_prompt::{input, INSTRUCTIONS};
use super::item_widget::ItemWidget;
use super::payload::{AllGameData, Player};
use crate::advice::{Advisor, Question, Trigger};

/// Advice on each death to an enemy champion, which may name the Item to build.
#[derive(Default)]
pub(super) struct DeathAdvice {
    /// `ChampionKill` events already seen, `None` before the game's first poll.
    seen_kills: Option<usize>,
    asking: Option<JoinHandle<Option<Answered>>>,
}

impl DeathAdvice {
    /// Deaths before the game's first poll, as when the app starts mid-game, ask nothing.
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        player: &Player,
        advisor: &Advisor,
        widget: &mut ItemWidget<'_>,
    ) {
        let kills: Vec<_> = data.events_named("ChampionKill").collect();
        let seen = self.seen_kills.replace(kills.len()).unwrap_or(kills.len());
        let death = kills
            .get(seen..)
            .unwrap_or_default()
            .iter()
            .rev()
            .find_map(|kill| Death::of(data, player, kill));
        if let Some(death) = death {
            self.ask(&death, advisor, widget.items);
        }
        self.collect(data, player, widget).await;
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.abort();
        }
    }

    fn ask(&mut self, death: &Death<'_>, advisor: &Advisor, items: &AdvisedItems) {
        // A new death means the player respawned since the last, whose answer is now late.
        self.end_game();
        let player = death.player;
        if !player.is_dead || player.respawn_timer <= 0.0 {
            return;
        }
        let Some(call) = advisor.call_for(Trigger::Death) else {
            return;
        };
        let to_build = items.to_build(player, death.enemies());
        let recommended = items.recommended(player);
        let input = input(call.language, death, to_build.as_ref(), &recommended);
        let ids: Vec<u32> = to_build.iter().map(|candidate| candidate.item).collect();
        let deadline = Instant::now() + Duration::from_secs_f64(player.respawn_timer);
        let advisor = advisor.clone();
        self.asking = Some(tokio::spawn(async move {
            let question = Question {
                trigger: Trigger::Death,
                instructions: INSTRUCTIONS,
                input: &input,
                candidates: &ids,
                deadline,
            };
            Answered::of(advisor.ask(&call, &question).await)
        }));
    }

    /// An answer that arrives once the player has respawned is dropped.
    async fn collect(&mut self, data: &AllGameData, player: &Player, widget: &mut ItemWidget<'_>) {
        let Some(asking) = self.asking.take_if(|asking| asking.is_finished()) else {
            return;
        };
        if !player.is_dead {
            return;
        }
        if let Some(answered) = asking.await.ok().flatten() {
            widget.offer(answered, Trigger::Death, data.game_time(), player);
        }
    }
}
