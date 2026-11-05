/// A poll at which the floored CS went up, the only kind at which the count is exact.
#[derive(Clone, Copy)]
pub(super) struct CsStep {
    pub(super) at: f64,
    pub(super) cs: u32,
}
