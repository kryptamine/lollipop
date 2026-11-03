use crate::benchmarks::ItemName;

/// An item earlier Advice in this game recommended, which the prompt names so the model
/// repeats it only when the game still calls for it.
pub(super) struct RecommendedItem<'a> {
    pub(super) item: u32,
    pub(super) name: Option<&'a ItemName>,
    pub(super) bought: bool,
}
