/// The item a model would put in place of the Next item, and why.
#[derive(Debug)]
pub(crate) struct ProposedChange {
    pub(crate) item: u32,
    pub(crate) reason: String,
}
