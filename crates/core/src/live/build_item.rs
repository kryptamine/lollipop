use serde::Serialize;

/// An item the Build Widget shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BuildItem {
    pub item: u32,
    /// `None` in a table built without a Data Dragon version.
    pub icon: Option<String>,
}
