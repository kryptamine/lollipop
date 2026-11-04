/// A held item a model would sell for an Item candidate once the inventory is full, and why.
#[derive(Debug)]
pub(crate) struct ProposedSwap {
    pub(crate) sell: u32,
    pub(crate) buy: u32,
    pub(crate) reason: String,
}
