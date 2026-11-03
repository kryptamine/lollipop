/// The items an Item candidate list may hold.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ItemClass {
    Completed,
    /// Tier-2 boots, which tier-3 boots build from.
    Boots,
}
