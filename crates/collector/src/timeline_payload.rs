//! The `match-v5` timeline JSON document, one struct per nested object.

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct TimelinePayload {
    pub(crate) info: TimelineInfo,
}

#[derive(Deserialize)]
pub(crate) struct TimelineInfo {
    pub(crate) frames: Vec<Frame>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Frame {
    pub(crate) timestamp: u64,
    pub(crate) participant_frames: HashMap<String, ParticipantFrame>,
    pub(crate) events: Vec<Event>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ParticipantFrame {
    pub(crate) participant_id: u32,
    pub(crate) minions_killed: u32,
    pub(crate) jungle_minions_killed: u32,
    pub(crate) total_gold: u32,
    pub(crate) level: u32,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub(crate) enum Event {
    #[serde(rename = "ELITE_MONSTER_KILL", rename_all = "camelCase")]
    EliteMonsterKill {
        timestamp: u64,
        monster_type: String,
        monster_sub_type: Option<String>,
    },
    #[serde(rename = "CHAMPION_KILL", rename_all = "camelCase")]
    ChampionKill {
        timestamp: u64,
        killer_id: u32,
        victim_id: u32,
        #[serde(default)]
        assisting_participant_ids: Vec<u32>,
    },
    #[serde(other)]
    Other,
}
