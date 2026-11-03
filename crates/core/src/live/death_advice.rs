use std::time::{Duration, Instant};

use tokio::task::JoinHandle;

use super::advice_slot::AdviceSlot;
use super::arrival::Arrival;
use super::death::Death;
use super::death_prompt::{input, INSTRUCTIONS};
use super::payload::{AllGameData, Player};
use super::recommended_item::RecommendedItem;
use crate::advice::{Advisor, Asked, Question, Trigger};
use crate::benchmarks::{BenchmarkTable, ItemTable};
use crate::game_inputs::{ClientInputs, GameInputs, Role};

/// Advice on each death to an enemy champion, which may name an Item candidate.
#[derive(Default)]
pub(super) struct DeathAdvice {
    /// Empty when the game's table is missing or Stale, so no item can be advised.
    items: ItemTable,
    role: Option<Role>,
    /// `ChampionKill` events already seen, `None` before the game's first poll.
    seen_kills: Option<usize>,
    asking: Option<JoinHandle<Option<Answered>>>,
    /// Items earlier Advice in this game showed.
    recommended: Vec<u32>,
}

struct Answered {
    text: String,
    item: Option<u32>,
}

impl DeathAdvice {
    pub(super) fn start(table: Option<&mut BenchmarkTable>, inputs: &GameInputs) -> Self {
        let patch = match &inputs.client {
            ClientInputs::Read(client) => client.patch,
            ClientInputs::Unavailable => None,
        };
        let items = table
            .filter(|table| patch.is_some_and(|patch| table.serves(patch)))
            .map(BenchmarkTable::take_items)
            .unwrap_or_default();
        Self {
            items,
            role: inputs.role,
            ..Self::default()
        }
    }

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
            self.ask(&death, advisor);
        }
        self.collect(data, player, slot).await;
    }

    pub(super) fn end_game(&mut self) {
        if let Some(asking) = self.asking.take() {
            asking.abort();
        }
    }

    fn ask(&mut self, death: &Death<'_>, advisor: &Advisor) {
        // A new death means the player respawned since the last, whose answer is now late.
        self.end_game();
        let player = death.player;
        if !player.is_dead || player.respawn_timer <= 0.0 {
            return;
        }
        let Some(call) = advisor.call_for(Trigger::Death) else {
            return;
        };
        let owned: Vec<u32> = player.items.iter().map(|item| item.id).collect();
        let candidates = match (self.role, player.cell_champion()) {
            (Some(role), Some(champion)) => {
                let enemies = death.enemies().filter_map(Player::cell_champion);
                self.items.candidates(champion, role, enemies, &owned)
            }
            _ => Vec::new(),
        };
        let recommended: Vec<RecommendedItem<'_>> = self
            .recommended
            .iter()
            .map(|&item| RecommendedItem {
                item,
                name: self.items.name_of(item),
                bought: owned.contains(&item),
            })
            .collect();
        let input = input(call.language, death, &candidates, &recommended);
        let ids: Vec<u32> = candidates.iter().map(|candidate| candidate.item).collect();
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
            match advisor.ask(&call, &question).await {
                Asked::Answered(answer) => {
                    let text = answer.advice.filter(|advice| !advice.trim().is_empty())?;
                    Some(Answered {
                        text,
                        item: answer.item,
                    })
                }
                Asked::Unavailable(text) => Some(Answered {
                    text: text.to_string(),
                    item: None,
                }),
                Asked::Nothing => None,
            }
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
        let Some(answered) = asking.await.ok().flatten() else {
            return;
        };
        let arrival = Arrival {
            trigger: Trigger::Death,
            text: answered.text,
            icon: answered.item.and_then(|item| self.items.icon_url(item)),
        };
        let shown = slot.offer(arrival, Some(data.game_time()));
        if let Some(item) = answered
            .item
            .filter(|item| shown && !self.recommended.contains(item))
        {
            self.recommended.push(item);
        }
    }
}
