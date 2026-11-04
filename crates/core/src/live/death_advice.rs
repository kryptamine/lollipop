use std::future::ready;

use super::build::Build;
use super::death::Death;
use super::death_prompt::{input, Around};
use super::event_name::EventName;
use super::fired::{AskInput, Fired, InputFuture};
use super::game_facts::GameFacts;
use super::health_trail::HealthTrail;
use super::payload::AllGameData;
use super::until::Until;
use crate::advice::Language;

/// Advice on a death to an enemy champion that teaches something, which may change the
/// Next item.
#[derive(Default)]
pub(super) struct DeathAdvice {
    /// `ChampionKill` events already seen, `None` before the game's first poll.
    seen_kills: Option<usize>,
    health: HealthTrail,
}

impl DeathAdvice {
    /// Deaths before the game's first poll, as when the app starts mid-game, ask nothing.
    pub(super) fn at<'a>(
        &mut self,
        data: &'a AllGameData,
        facts: &'a GameFacts<'a>,
    ) -> Option<Fired<impl AskInput + 'a>> {
        let player = data.active_player()?;
        let kills: Vec<_> = data.events_named(EventName::ChampionKill).collect();
        let seen = self.seen_kills.replace(kills.len()).unwrap_or(kills.len());
        let death = kills
            .get(seen..)
            .unwrap_or_default()
            .iter()
            .rev()
            .find_map(|kill| Death::of(data, player, kill));
        let fired = death.map(|death| {
            // A new death means the player respawned since the last, whose answer is now late.
            if !(death.teaches() && player.is_dead && player.respawn_timer > 0.0) {
                return Fired::Drop;
            }
            // The trail as it stood before this poll, whose death clears it.
            let health = self.health.clone();
            Fired::Ask {
                until: Until::Respawn,
                input: move |language: Language, build: &Build| -> InputFuture {
                    let around = Around {
                        build,
                        facts,
                        health: &health,
                    };
                    Box::pin(ready(input(language, &death, &around)))
                },
            }
        });
        self.health.observe(data, player.is_dead);
        fired
    }
}
