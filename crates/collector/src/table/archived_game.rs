//! The archived `match-v5` match JSON document, as far as Item candidates read it.

use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct ArchivedGame {
    pub(super) info: GameInfo,
}

#[derive(Deserialize)]
pub(super) struct GameInfo {
    pub(super) participants: Vec<GameParticipant>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GameParticipant {
    pub(super) participant_id: u32,
    pub(super) champion_name: String,
    pub(super) team_position: String,
    pub(super) team_id: u32,
    pub(super) magic_damage_dealt_to_champions: u64,
    pub(super) total_damage_dealt_to_champions: u64,
}
