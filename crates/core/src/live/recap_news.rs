use super::recap::Recap;

/// What one poll tells the app of the Recap.
#[derive(Debug)]
pub struct RecapNews {
    /// The Recap when it was just made or its Lessons just arrived.
    pub recap: Option<Recap>,
    /// True once, on the poll at which the Recap window should open.
    pub open: bool,
}
