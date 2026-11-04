use super::gold_earned::ItemCounts;

/// The Next item, and the items held when it took the lead.
pub(super) struct Lead {
    pub(super) item: u32,
    held_then: ItemCounts,
}

impl Lead {
    pub(super) const fn new(item: u32, held_then: ItemCounts) -> Self {
        Self { item, held_then }
    }

    /// Whether one of `components` was bought since it took the lead: a second Long Sword
    /// counts, though the player held one already.
    pub(super) fn started(&self, components: &[u32], held: &ItemCounts) -> bool {
        components.iter().any(|component| {
            let count = |counts: &ItemCounts| counts.get(component).copied().unwrap_or(0);
            count(held) > count(&self.held_then)
        })
    }
}
