use serde::Serialize;

use super::build_item::BuildItem;

/// A Swap as the Build Widget shows it: `sell → buy`, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShownSwap {
    pub sell: BuildItem,
    pub buy: BuildItem,
    pub reason: String,
}
