use super::payload::Item;
use crate::benchmarks::ItemCandidate;

/// What a Swap may sell and buy once the inventory is full.
pub(super) struct SwapOptions<'a> {
    pub(super) sells: Vec<&'a Item>,
    pub(super) buys: Vec<&'a ItemCandidate>,
}
