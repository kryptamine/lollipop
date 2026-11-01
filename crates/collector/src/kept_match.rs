use std::collections::BTreeMap;

use lollipop_core::game_inputs::Patch;

use crate::match_payload::MatchPayload;
use crate::store::PendingMatch;
use crate::timeline_payload::{Event, TimelinePayload};

const ELDER_DRAGON: &str = "ELDER_DRAGON";

/// The compact form of a match. It has no field for a PUUID, which is never stored.
pub(crate) struct KeptMatch {
    pub(crate) match_id: String,
    pub(crate) tier: String,
    pub(crate) patch: Patch,
    pub(crate) participants: Vec<KeptParticipant>,
    pub(crate) minute_values: Vec<MinuteValues>,
    pub(crate) first_kills: BTreeMap<String, u64>,
}

pub(crate) struct KeptParticipant {
    pub(crate) id: u32,
    pub(crate) champion: String,
    pub(crate) role: String,
}

/// One timeline frame of one participant. Frames run late by up to a second, so the
/// frame's own time is kept for aligning them to the minute.
pub(crate) struct MinuteValues {
    pub(crate) participant: u32,
    pub(crate) at_ms: u64,
    pub(crate) cs: u32,
    pub(crate) total_gold: u32,
    pub(crate) level: u32,
}

impl KeptMatch {
    pub(crate) fn new(
        pending: &PendingMatch,
        patch: Patch,
        game: MatchPayload,
        timeline: &TimelinePayload,
    ) -> Self {
        let participants = game
            .info
            .participants
            .into_iter()
            .map(|participant| KeptParticipant {
                id: participant.participant_id,
                champion: participant.champion_name,
                role: participant.team_position,
            });
        Self {
            match_id: pending.match_id.clone(),
            tier: pending.tier.clone(),
            patch,
            participants: participants.collect(),
            minute_values: minute_values(timeline),
            first_kills: first_kills(timeline),
        }
    }
}

fn minute_values(timeline: &TimelinePayload) -> Vec<MinuteValues> {
    let mut values = Vec::new();
    for frame in &timeline.info.frames {
        values.extend(
            frame
                .participant_frames
                .values()
                .map(|participant| MinuteValues {
                    participant: participant.participant_id,
                    at_ms: frame.timestamp,
                    cs: participant.minions_killed + participant.jungle_minions_killed,
                    total_gold: participant.total_gold,
                    level: participant.level,
                }),
        );
    }
    values
}

/// Elder has a schedule of its own, so it counts apart from the elemental drakes.
fn first_kills(timeline: &TimelinePayload) -> BTreeMap<String, u64> {
    let mut kills: BTreeMap<String, u64> = BTreeMap::new();
    let events = timeline.info.frames.iter().flat_map(|frame| &frame.events);
    for event in events {
        if let Event::EliteMonsterKill {
            timestamp,
            monster_type,
            monster_sub_type,
        } = event
        {
            let objective = match monster_sub_type.as_deref() {
                Some(ELDER_DRAGON) => ELDER_DRAGON,
                _ => monster_type,
            };
            let first = kills.entry(objective.to_string()).or_insert(*timestamp);
            *first = (*first).min(*timestamp);
        }
    }
    kills
}
