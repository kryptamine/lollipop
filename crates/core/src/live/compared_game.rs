use super::comparison::Comparison;
use super::held_marker::HeldMarker;
use super::markers::Markers;
use super::recent_cs::RecentCs;
use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use crate::benchmarks::{BenchmarkTable, BenchmarksAt, CellBenchmarks};
use crate::game_inputs::{ClientInputs, GameInputs, Role, TierReading};

// Draft, ranked solo, blind and ranked flex. Swiftplay, Practice Tool and anything
// unknown fall outside, and that is no fault worth a notice.
pub(super) const MARKER_QUEUES: [u32; 4] = [400, 420, 430, 440];

// The API floors CS to 10, so the true value is on average 5 above it.
pub(super) const CS_OFFSET: f64 = 5.0;
const SECONDS_PER_MINUTE: f64 = 60.0;
const ZONE_SHARE: f64 = 0.08;
const CS_ZONE_FLOOR: f64 = 8.0;
const GOLD_ZONE_FLOOR: f64 = 150.0;
const LEVEL_ZONE: f64 = 1.0;
// On kills and assists less deaths; not yet tested against real spread.
const SCORES_ZONE: f64 = 2.0;
// In CS a minute. The 10-CS floor alone moves a three-minute pace by up to 1.7, so the
// floor stays above it; not yet tested against real spread.
const PACE_ZONE_SHARE: f64 = 0.2;
const PACE_ZONE_FLOOR: f64 = 2.0;

pub(super) struct ComparedGame {
    cell: CellBenchmarks,
    has_cs_marker: bool,
    cs: HeldMarker,
    gold: HeldMarker,
    level: HeldMarker,
    scores: HeldMarker,
    cs_pace: HeldMarker,
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
            scores: HeldMarker::default(),
            cs_pace: HeldMarker::default(),
        })
    }

    pub(super) fn at(
        &mut self,
        game_time: f64,
        snapshot: &Snapshot,
        recent: Option<RecentCs>,
    ) -> (Markers, ShownBenchmarks) {
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
        let score_benchmarks = benchmarks.and_then(|b| b.scores);
        let scores = score_benchmarks
            .zip(snapshot.scores)
            .map(|(b, scores)| Comparison {
                value: scores.net(),
                benchmark: b.net(),
                neutral_zone: SCORES_ZONE,
            });
        let cs_pace = cs_benchmarks
            .zip(recent)
            .and_then(|(now, recent)| self.pace_comparison(game_time, &now, recent));
        let markers = Markers {
            cs: self.cs.update(cs),
            gold: self.gold.update(gold),
            level: self.level.update(level),
            scores: self.scores.update(scores),
            cs_pace: self.cs_pace.update(cs_pace),
        };
        let behind = self.cell.at_minute_behind(game_time);
        let shown = ShownBenchmarks {
            cs: cs_benchmarks.and(behind).map(|b| b.cs),
            gold: gold_benchmarks.and(behind).map(|b| b.gold),
            level: benchmarks.and(behind).map(|b| b.level),
            scores: score_benchmarks
                .and(snapshot.scores)
                .and(behind)
                .and_then(|b| b.scores),
            cs_pace: cs_pace.map(|pace| pace.benchmark),
        };
        (markers, shown)
    }

    /// The player's CS a minute against what the Cell gains over the same stretch.
    fn pace_comparison(
        &self,
        game_time: f64,
        now: &BenchmarksAt,
        recent: RecentCs,
    ) -> Option<Comparison> {
        let then = self.cell.at(recent.since)?;
        let benchmark = (now.cs - then.cs) / ((game_time - recent.since) / SECONDS_PER_MINUTE);
        Some(Comparison {
            value: recent.per_minute,
            benchmark,
            neutral_zone: (benchmark * PACE_ZONE_SHARE).max(PACE_ZONE_FLOOR),
        })
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
