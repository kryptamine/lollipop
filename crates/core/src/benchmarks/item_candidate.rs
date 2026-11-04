/// An Item candidate the Build holds.
#[derive(Debug)]
pub(crate) struct ItemCandidate {
    pub(crate) item: u32,
    pub(crate) boots: bool,
    /// The transforms that hold `item`.
    pub(super) owned_as: Vec<u32>,
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
