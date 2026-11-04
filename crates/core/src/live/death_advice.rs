use std::future::ready;

use super::advice_ask::AdviceAsk;
use super::advice_slot::AdviceSlot;
use super::build::Build;
use super::death::Death;
use super::death_prompt::{input, INSTRUCTIONS};
use super::event_name::EventName;
use super::payload::AllGameData;
use super::until::Until;
use crate::advice::{Advisor, Trigger};

/// Advice on a death to an enemy champion that teaches something, which may change the
/// Next item.
pub(super) struct DeathAdvice {
    /// `ChampionKill` events already seen, `None` before the game's first poll.
    seen_kills: Option<usize>,
    ask: AdviceAsk,
}

impl DeathAdvice {
    pub(super) fn new(advisor: &Advisor) -> Self {
        Self {
            seen_kills: None,
            ask: AdviceAsk::new(Trigger::Death, INSTRUCTIONS, advisor),
        }
    }

    /// Deaths before the game's first poll, as when the app starts mid-game, ask nothing.
    pub(super) async fn at(
        &mut self,
        data: &AllGameData,
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
            if death.teaches() && player.is_dead && player.respawn_timer > 0.0 {
                self.ask
                    .ask(data, build, Until::Respawn, |language, build| {
                        ready(input(language, &death, build))
                    });
            }
        }
        self.ask
            .collect(data, Some(data.game_time()), slot, build)
            .await;
    }

    pub(super) fn end_game(&mut self) {
        self.ask.end();
    }
}
