use std::time::{Duration, Instant};

use super::advice_slot::AdviceSlot;
use super::arrival::Arrival;
use super::build::Build;
use super::death::Death;
use super::death_prompt::{input, INSTRUCTIONS};
use super::event_name::EventName;
use super::payload::{AllGameData, Player};
use super::pending_answer::PendingAnswer;
use crate::advice::{Advisor, AnswerParts, Question, Trigger};

/// Advice on a death to an enemy champion that teaches something, which may change the
/// Next item.
#[derive(Default)]
pub(super) struct DeathAdvice {
    /// `ChampionKill` events already seen, `None` before the game's first poll.
    seen_kills: Option<usize>,
    asking: PendingAnswer<AnswerParts>,
}

impl DeathAdvice {
    /// Deaths before the game's first poll, as when the app starts mid-game, ask nothing.
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
        advisor: &Advisor,
        slot: &mut AdviceSlot,
        build: &mut Build,
    ) {
        let Some(player) = data.active_player() else {
            return;
        };
        let kills: Vec<_> = data.events_named(EventName::ChampionKill).collect();
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
                self.ask(&death, advisor, build);
            }
        }
        self.collect(data, player, slot, build).await;
    }

    pub(super) fn end_game(&mut self) {
        self.asking = PendingAnswer::default();
    }

    fn ask(&mut self, death: &Death<'_>, advisor: &Advisor, build: &mut Build) {
        let player = death.player;
        if !player.is_dead || player.respawn_timer <= 0.0 {
            return;
        }
        let Some(call) = advisor.call_for(Trigger::Death) else {
            return;
        };
        build.track_lead(player);
        let input = input(call.language, death, build);
        let deadline = Instant::now() + Duration::from_secs_f64(player.respawn_timer);
        let advisor = advisor.clone();
        self.asking = PendingAnswer::spawn(async move {
            let question = Question {
                trigger: Trigger::Death,
                instructions: INSTRUCTIONS,
                input: &input,
                deadline,
            };
            advisor.ask(&call, &question).await.into_parts()
        });
    }

    /// An answer that arrives once the player has respawned is dropped, Build change too.
    async fn collect(
        &mut self,
        data: &AllGameData,
        player: &Player,
        slot: &mut AdviceSlot,
        build: &mut Build,
    ) {
        let Some(AnswerParts {
            text,
            change,
            swap,
            elixir,
        }) = self.asking.finished().await
        else {
            return;
        };
        if !player.is_dead {
            return;
        }
        if let Some(change) = change {
            build.change(change, player, Some(data.game_time()));
        }
        if let Some(swap) = swap {
            build.swap(swap, player);
        }
        if let Some(elixir) = elixir {
            build.pick_elixir(elixir, player);
        }
        if let Some(text) = text {
            let arrival = Arrival {
                trigger: Trigger::Death,
                text,
            };
            slot.offer(arrival, Some(data.game_time()));
        }
    }
}
