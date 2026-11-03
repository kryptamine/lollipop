//! The archived `match-v5` timeline JSON document, as far as Item candidates read it.
//! Riot documents no event's fields: these shapes come from the pilot's data.

use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct ArchivedTimeline {
    pub(super) info: TimelineInfo,
}

#[derive(Deserialize)]
pub(super) struct TimelineInfo {
    pub(super) frames: Vec<TimelineFrame>,
}

#[derive(Deserialize)]
pub(super) struct TimelineFrame {
    pub(super) events: Vec<ItemEvent>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub(super) enum ItemEvent {
    #[serde(rename = "ITEM_PURCHASED", rename_all = "camelCase")]
    Purchased {
        timestamp: u64,
        participant_id: u32,
        item_id: u32,
    },
    /// Undoes a purchase when `after_id` is 0, and a sale or a swap otherwise.
    #[serde(rename = "ITEM_UNDO", rename_all = "camelCase")]
    Undo {
        participant_id: u32,
        before_id: u32,
        after_id: u32,
    },
    #[serde(other)]
    Other,
}
