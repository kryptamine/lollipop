use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use super::spawn::Spawn;

/// What the overlay worked out of the game at this poll beyond the payload, which a
/// model could not: the player against their Benchmarks, and the objectives to come.
pub(super) struct GameFacts<'a> {
    pub(super) snapshot: &'a Snapshot,
    pub(super) benchmarks: &'a ShownBenchmarks,
    /// Soonest first, empty in a queue with no Objective schedule.
    pub(super) spawns: Vec<Spawn>,
}
