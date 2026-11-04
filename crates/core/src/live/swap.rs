use crate::benchmarks::ItemCandidate;

/// The Swap a model proposed, held until its item to buy is.
pub(super) struct Swap {
    pub(super) sell: u32,
    pub(super) buy: ItemCandidate,
    pub(super) reason: String,
}
