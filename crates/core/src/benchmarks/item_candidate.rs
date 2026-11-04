/// An Item candidate: `share` of the Cell's games bought it, first at the median `minute`.
#[derive(Debug, Clone)]
pub(crate) struct ItemCandidate {
    pub(crate) item: u32,
    pub(crate) boots: bool,
    pub(crate) share: f64,
    pub(crate) minute: f64,
    /// The transforms that hold `item`.
    pub(super) owned_as: Vec<u32>,
    /// Every component down its build tree; empty in a table built before they were listed.
    pub(crate) builds_from: Vec<u32>,
}

impl ItemCandidate {
    /// Whether `item` is this candidate, itself or transformed.
    pub(crate) fn is(&self, item: u32) -> bool {
        self.item == item || self.owned_as.contains(&item)
    }

    pub(crate) fn held(&self, owned: &[u32]) -> bool {
        owned.iter().any(|&item| self.is(item))
    }
}
