use super::first_purchase::FirstPurchase;

/// What one participant of an archived match gives Item candidates.
pub(super) struct ParticipantItems {
    pub(super) champion: String,
    pub(super) role: String,
    pub(super) team: u32,
    pub(super) magic_damage: u64,
    pub(super) total_damage: u64,
    pub(super) purchases: Vec<FirstPurchase>,
}
