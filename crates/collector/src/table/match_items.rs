use std::collections::HashMap;

use lollipop_core::game_inputs::Patch;
use serde::de::DeserializeOwned;

use super::archived_game::ArchivedGame;
use super::archived_timeline::{ArchivedTimeline, ItemEvent};
use super::first_purchase::FirstPurchase;
use super::item_class::ItemClass;
use super::participant_items::ParticipantItems;
use super::stored_payloads::StoredPayloads;
use super::table_error::TableError;

/// One archived match, as Item candidates and the Damage and Threats read it.
pub(super) struct MatchItems {
    pub(super) patch: Patch,
    pub(super) minutes: f64,
    pub(super) participants: Vec<ParticipantItems>,
}

impl MatchItems {
    pub(super) fn read(
        stored: &StoredPayloads,
        classes: &HashMap<u32, ItemClass>,
    ) -> Result<Self, TableError> {
        let game: ArchivedGame = unpacked(stored, &stored.game)?;
        let timeline: ArchivedTimeline = unpacked(stored, &stored.timeline)?;
        let mut purchases = net_purchases(&timeline);
        let participants = game.info.participants.into_iter().map(|participant| {
            let bought = purchases
                .remove(&participant.participant_id)
                .unwrap_or_default();
            ParticipantItems {
                champion: participant.champion_name,
                role: participant.team_position,
                team: participant.team_id,
                magic_damage: participant.magic_damage_dealt_to_champions,
                total_damage: participant.total_damage_dealt_to_champions,
                heal: participant.total_heal,
                mitigated: participant.damage_self_mitigated,
                purchases: first_purchases(&bought, classes),
            }
        });
        Ok(Self {
            patch: stored.patch,
            minutes: seconds(game.info.game_duration) / 60.0,
            participants: participants.collect(),
        })
    }

    /// Each team id once, in seat order.
    pub(super) fn teams(&self) -> impl Iterator<Item = u32> + '_ {
        let mut seen: Vec<u32> = Vec::new();
        self.participants.iter().filter_map(move |participant| {
            let new = !seen.contains(&participant.team);
            seen.push(participant.team);
            new.then_some(participant.team)
        })
    }
}

fn unpacked<T: DeserializeOwned>(stored: &StoredPayloads, body: &[u8]) -> Result<T, TableError> {
    let refuse = |reason: String| TableError::Archive(stored.match_id.clone(), reason);
    let json = zstd::decode_all(body).map_err(|error| refuse(error.to_string()))?;
    serde_json::from_slice(&json).map_err(|error| refuse(error.to_string()))
}

/// Each participant's purchases in time order, less the ones undone in the shop.
fn net_purchases(timeline: &ArchivedTimeline) -> HashMap<u32, Vec<(u32, u64)>> {
    let mut purchases: HashMap<u32, Vec<(u32, u64)>> = HashMap::new();
    let events = timeline.info.frames.iter().flat_map(|frame| &frame.events);
    for event in events {
        match *event {
            ItemEvent::Purchased {
                timestamp,
                participant_id,
                item_id,
            } => {
                purchases
                    .entry(participant_id)
                    .or_default()
                    .push((item_id, timestamp));
            }
            ItemEvent::Undo {
                participant_id,
                before_id,
                after_id: 0,
            } => {
                let bought = purchases.entry(participant_id).or_default();
                if let Some(undone) = bought.iter().rposition(|(item, _)| *item == before_id) {
                    bought.remove(undone);
                }
            }
            _ => {}
        }
    }
    purchases
}

fn first_purchases(bought: &[(u32, u64)], classes: &HashMap<u32, ItemClass>) -> Vec<FirstPurchase> {
    let mut firsts: Vec<FirstPurchase> = Vec::new();
    for &(item, at_ms) in bought {
        let Some(&class) = classes.get(&item) else {
            continue;
        };
        if firsts.iter().all(|first| first.item != item) {
            firsts.push(FirstPurchase { item, class, at_ms });
        }
    }
    firsts
}

#[allow(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a game lasts far below 2^52 seconds"
)]
const fn seconds(value: u64) -> f64 {
    value as f64
}
