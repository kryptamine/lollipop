//! The `match-v5` match JSON document, one struct per nested object.

use lollipop_core::game_inputs::Patch;
use serde::Deserialize;

pub(crate) const RANKED_SOLO_QUEUE: u32 = 420;
const MIN_DURATION_S: u64 = 15 * 60;

#[derive(Deserialize)]
pub(crate) struct MatchPayload {
    pub(crate) info: MatchInfo,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MatchInfo {
    game_duration: u64,
    game_version: String,
    queue_id: u32,
    pub(crate) participants: Vec<MatchParticipant>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MatchParticipant {
    pub(crate) participant_id: u32,
    pub(crate) champion_name: String,
    pub(crate) team_position: String,
    game_ended_in_early_surrender: bool,
}

impl MatchPayload {
    /// The patch of a game worth a timeline call: `endOfGameResult` does not remove early
    /// games, and a patch below `window_floor` has left the Patch window.
    pub(crate) fn patch_if_kept(&self, window_floor: Option<Patch>) -> Option<Patch> {
        let info = &self.info;
        let early_surrender = info
            .participants
            .iter()
            .any(|participant| participant.game_ended_in_early_surrender);
        if info.queue_id != RANKED_SOLO_QUEUE
            || info.game_duration < MIN_DURATION_S
            || early_surrender
        {
            return None;
        }
        Patch::parse(&info.game_version)
            .filter(|patch| window_floor.is_none_or(|floor| *patch >= floor))
    }
}
