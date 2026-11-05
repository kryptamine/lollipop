use super::compared_game::ComparedGame;
use super::markers::Markers;
use super::recent_cs::RecentCs;
use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use crate::benchmarks::BenchmarkTable;
use crate::game_inputs::GameInputs;

/// One game's Markers and their Benchmarks. The table is read once, as the game starts,
/// and only the player's Cell is kept, so a new file on disk cannot change what a Marker
/// means mid-game.
#[derive(Default)]
pub(super) struct GameMarkers {
    compared: Option<ComparedGame>,
}

impl GameMarkers {
    pub(super) fn start(
        table: Option<BenchmarkTable>,
        inputs: &GameInputs,
        champion: Option<&str>,
    ) -> Self {
        Self {
            compared: ComparedGame::start(table, inputs, champion),
        }
    }

    pub(super) fn at(
        &mut self,
        game_time: f64,
        snapshot: &Snapshot,
        recent: Option<RecentCs>,
    ) -> (Markers, ShownBenchmarks) {
        self.compared
            .as_mut()
            .map_or_else(Default::default, |game| {
                game.at(game_time, snapshot, recent)
            })
    }
}
