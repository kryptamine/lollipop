use super::item_class::ItemClass;

/// A player's first purchase of an item a candidate list may hold, net of undone ones.
pub(super) struct FirstPurchase {
    pub(super) item: u32,
    pub(super) class: ItemClass,
    pub(super) at_ms: u64,
}
