use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use super::advice_slot::AdviceSlot;
use super::arrival::Arrival;
use super::death::Death;
use super::death_prompt::{input, INSTRUCTIONS};
use super::payload::{AllGameData, Player};
use crate::advice::{Advisor, Question, Trigger};

/// Advice on a death to an enemy champion that teaches something.
#[derive(Default)]
pub(super) struct DeathAdvice {
    /// `ChampionKill` events already seen, `None` before the game's first poll.
    seen_kills: Option<usize>,
    asking: Option<JoinHandle<Option<String>>>,
}

impl DeathAdvice {
    /// Deaths before the game's first poll, as when the app starts mid-game, ask nothing.
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        player: &Player,
        advisor: &Advisor,
        slot: &mut AdviceSlot,
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
            // A new death means the player respawned since the last, whose answer is now late.
            self.end_game();
            if death.teaches() {
                self.ask(&death, advisor);
            }
        }
        self.collect(data, player, slot).await;
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.abort();
        }
    }

    fn ask(&mut self, death: &Death<'_>, advisor: &Advisor) {
        let player = death.player;
        if !player.is_dead || player.respawn_timer <= 0.0 {
            return;
        }
        let Some(call) = advisor.call_for(Trigger::Death) else {
            return;
        };
        let input = input(call.language, death);
        let deadline = Instant::now() + Duration::from_secs_f64(player.respawn_timer);
        let advisor = advisor.clone();
        self.asking = Some(tokio::spawn(async move {
            let question = Question {
                trigger: Trigger::Death,
                instructions: INSTRUCTIONS,
                input: &input,
                deadline,
            };
            advisor.ask(&call, &question).await.into_text()
        }));
    }

    /// An answer that arrives once the player has respawned is dropped.
    async fn collect(&mut self, data: &AllGameData, player: &Player, slot: &mut AdviceSlot) {
        let Some(asking) = self.asking.take_if(|asking| asking.is_finished()) else {
            return;
        };
        if !player.is_dead {
            return;
        }
        if let Some(text) = asking.await.ok().flatten() {
            let arrival = Arrival {
                trigger: Trigger::Death,
                text,
            };
            slot.offer(arrival, Some(data.game_time()));
        }
    }
}
