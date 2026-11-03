/// An item Advice showed, with the completed items the player held then.
pub(super) struct ShownItem {
    pub(super) item: u32,
    pub(super) finished_then: Vec<u32>,
}

impl ShownItem {
    /// Whether the player has since finished another item, and so left this one.
    pub(super) fn passed_over(&self, finished: &[u32]) -> bool {
        finished
            .iter()
            .any(|item| *item != self.item && !self.finished_then.contains(item))
    }
}
