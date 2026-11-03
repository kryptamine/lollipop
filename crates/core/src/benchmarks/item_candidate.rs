use super::item_name::ItemName;

/// An item the Cell's games bought: `share` of them, first at the median `minute`.
#[derive(Debug)]
pub(crate) struct ItemCandidate<'a> {
    pub(crate) item: u32,
    pub(crate) share: f64,
    pub(crate) minute: f64,
    pub(crate) name: Option<&'a ItemName>,
}
