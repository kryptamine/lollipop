use super::comparison::Comparison;
use super::held_marker::HeldMarker;
use super::markers::Markers;
use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use crate::benchmarks::{BenchmarkTable, BenchmarksAt, CellBenchmarks};
use crate::game_inputs::{ClientInputs, GameInputs, Role, TierReading};

// Draft, ranked solo, blind and ranked flex. Swiftplay, Practice Tool and anything
// unknown fall outside, and that is no fault worth a notice.
pub(super) const MARKER_QUEUES: [u32; 4] = [400, 420, 430, 440];

// The API floors CS to 10, so the true value is on average 5 above it.
const CS_OFFSET: f64 = 5.0;
const ZONE_SHARE: f64 = 0.08;
const CS_ZONE_FLOOR: f64 = 8.0;
const GOLD_ZONE_FLOOR: f64 = 150.0;
const LEVEL_ZONE: f64 = 1.0;

pub(super) struct ComparedGame {
    cell: CellBenchmarks,
    has_cs_marker: bool,
    cs: HeldMarker,
    gold: HeldMarker,
    level: HeldMarker,
}

impl ComparedGame {
    pub(super) fn start(
        table: Option<BenchmarkTable>,
        inputs: &GameInputs,
        champion: Option<&str>,
    ) -> Option<Self> {
        let ClientInputs::Read(client) = &inputs.client else {
            return None;
        };
        let TierReading::Ranked(tier) = client.tier else {
            return None;
        };
        let patch = client.patch?;
        if !MARKER_QUEUES.contains(&client.queue?) {
            return None;
        }
        let role = inputs.role?;
        let cell = table
            .filter(|table| table.serves(patch))?
            .into_cell(champion?, role, tier)?;
        Some(Self {
            cell,
            has_cs_marker: role != Role::Support,
            cs: HeldMarker::default(),
            gold: HeldMarker::default(),
            level: HeldMarker::default(),
        })
    }

    pub(super) fn at(&mut self, game_time: f64, snapshot: &Snapshot) -> (Markers, ShownBenchmarks) {
        let benchmarks = self.cell.at(game_time);
        let cs_benchmarks = benchmarks.filter(|_| self.has_cs_marker);
        let gold_benchmarks = benchmarks.filter(|_| snapshot.gold_earned.is_some());
        let cs = cs_benchmarks.map(|b| cs_comparison(snapshot.cs, &b));
        let gold = gold_benchmarks
            .zip(snapshot.gold_earned)
            .map(|(b, gold)| gold_comparison(gold, &b));
        let level = benchmarks.map(|b| Comparison {
            value: f64::from(snapshot.level),
            benchmark: b.level,
            neutral_zone: LEVEL_ZONE,
        });
        let markers = Markers {
            cs: self.cs.update(cs),
            gold: self.gold.update(gold),
            level: self.level.update(level),
        };
        let shown = ShownBenchmarks {
            cs: cs_benchmarks.map(|b| b.cs),
            gold: gold_benchmarks.map(|b| b.gold),
            level: benchmarks.map(|b| b.level),
        };
        (markers, shown)
    }
}

fn cs_comparison(cs: u32, benchmarks: &BenchmarksAt) -> Comparison {
    Comparison {
        value: f64::from(cs) + CS_OFFSET,
        benchmark: benchmarks.cs,
        neutral_zone: (benchmarks.cs * ZONE_SHARE).max(CS_ZONE_FLOOR),
    }
}

fn gold_comparison(gold: u32, benchmarks: &BenchmarksAt) -> Comparison {
    Comparison {
        value: f64::from(gold),
        benchmark: benchmarks.gold,
        neutral_zone: (benchmarks.gold * ZONE_SHARE).max(GOLD_ZONE_FLOOR),
    }
}
