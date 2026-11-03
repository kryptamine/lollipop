/// Where a candidate stands against earlier Advice, best first.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Standing {
    /// Shown and still the plan, the latest shown first.
    InProgress(usize),
    NeverShown,
    PassedOver,
}
