use serde::Serialize;

/// Games a Trend shows, the latest last.
pub(super) const TREND_GAMES: u32 = 10;
const TEN_MINUTES: f64 = 600.0;
// League is polled each second, so a first poll later than this is a game joined late
// or a lag spike, whose CS would flatter or wrong the player.
const LATEST_READING: f64 = 605.0;

/// One game of a Trend: the player's CS at 10:00 against the Cell's at that minute.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendGame {
    pub cs: u32,
    /// `None` where the game had no CS Benchmark, as for a support.
    pub benchmark: Option<f64>,
}

impl TrendGame {
    /// `None` unless `game_time` is just past 10:00.
    pub(super) fn read_at(game_time: f64, cs: u32, benchmark: Option<f64>) -> Option<Self> {
        (TEN_MINUTES..LATEST_READING)
            .contains(&game_time)
            .then_some(Self { cs, benchmark })
    }
}
