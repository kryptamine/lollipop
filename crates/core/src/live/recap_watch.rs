use std::sync::Arc;

use super::event_name::EventName;
use super::lessons::Lessons;
use super::marker::Marker;
use super::payload::{AllGameData, Event};
use super::prompt_lines::champion;
use super::recap::Recap;
use super::recap_minute::RecapMinute;
use super::shown_benchmarks::ShownBenchmarks;
use super::snapshot::Snapshot;
use super::stretch::Stretch;
use super::trend_game::TrendGame;
use crate::game_inputs::{GameInputs, Tier};

// Fewer minutes seen, as in a remake or a game joined near its end, chart nothing
// worth a window.
const MINIMUM_MINUTES: usize = 5;
const SECONDS_PER_MINUTE: f64 = 60.0;

/// One game as its Recap will tell it, recorded poll by poll.
pub(super) struct RecapWatch {
    inputs: Arc<GameInputs>,
    minutes: Vec<RecapMinute>,
    slow_farm: Vec<Stretch>,
    /// Where the CS pace went below, while it still is.
    slow_since: Option<f64>,
    /// The first reading just past 10:00, `None` for a game joined after it.
    at_ten: Option<TrendGame>,
    /// The last running poll, read when no poll sees `GameEnd`.
    last: Option<AllGameData>,
}

impl RecapWatch {
    pub(super) const fn start(inputs: Arc<GameInputs>) -> Self {
        Self {
            inputs,
            minutes: Vec::new(),
            slow_farm: Vec::new(),
            slow_since: None,
            at_ten: None,
            last: None,
        }
    }

    pub(super) fn observe(
        &mut self,
        game_time: f64,
        snapshot: &Snapshot,
        benchmarks: &ShownBenchmarks,
        cs_pace: Option<Marker>,
    ) {
        let minute = whole_minute(game_time);
        if self.minutes.last().is_none_or(|last| last.minute < minute) {
            self.minutes.push(RecapMinute {
                minute,
                cs: snapshot.cs,
                gold_earned: snapshot.gold_earned,
                cs_benchmark: benchmarks.cs,
                gold_benchmark: benchmarks.gold,
            });
        }
        if self.at_ten.is_none() {
            self.at_ten = TrendGame::read_at(game_time, snapshot.cs, benchmarks.cs);
        }
        if cs_pace == Some(Marker::Below) {
            self.slow_since.get_or_insert(game_time);
        } else if let Some(from) = self.slow_since.take() {
            self.slow_farm.push(Stretch {
                from,
                to: game_time,
            });
        }
    }

    pub(super) fn keep(&mut self, data: AllGameData) {
        self.last = Some(data);
    }

    pub(super) const fn last(&self) -> Option<&AllGameData> {
        self.last.as_ref()
    }

    pub(super) const fn at_ten(&self) -> Option<TrendGame> {
        self.at_ten
    }

    pub(super) fn tier(&self) -> Option<Tier> {
        self.inputs.ranked_tier()
    }

    /// `None` for a game seen too little, or with no player in it. Its Trend and Lessons
    /// are left to the caller.
    pub(super) fn recap(&self, data: &AllGameData) -> Option<Recap> {
        let player = data.active_player()?;
        let length = data.game_time();
        if self.minutes.len() < MINIMUM_MINUTES {
            return None;
        }
        let deaths = data.deaths_of(player).filter_map(Event::time).collect();
        let open = self.slow_since.map(|from| Stretch { from, to: length });
        Some(Recap {
            champion: champion(player).to_string(),
            role: self.inputs.role,
            outcome: data
                .events_named(EventName::GameEnd)
                .find_map(Event::outcome),
            length,
            scores: player.scores.player_scores(),
            minutes: self.minutes.clone(),
            deaths,
            slow_farm: self.slow_farm.iter().copied().chain(open).collect(),
            trend: Vec::new(),
            lessons: Lessons::Asking,
        })
    }
}

#[allow(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "game time is a few thousand seconds, never negative"
)]
fn whole_minute(game_time: f64) -> u32 {
    (game_time.max(0.0) / SECONDS_PER_MINUTE).floor() as u32
}
