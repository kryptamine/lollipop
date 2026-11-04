use crate::benchmarks::ItemCandidate;

/// The Next item while it is not started, and the Item candidates that may take its place.
pub(super) struct OpenNext<'a> {
    pub(super) next: &'a ItemCandidate,
    pub(super) replacements: Vec<&'a ItemCandidate>,
}
