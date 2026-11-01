//! Polls Riot's Live Client Data API for the active player's own stats.

mod clock;
mod compared_game;
mod dragon_pit;
mod game_hints;
mod game_markers;
mod gold_earned;
mod held_marker;
mod inputs_of_game;
mod live_game;
mod marker;
mod markers;
mod objective_schedule;
mod payload;
mod poll;
mod snapshot;
mod spawn;
mod status_line;
mod team;
mod timing_hint;

pub use clock::{Clock, SystemClock};
pub use live_game::LiveGame;
pub use marker::Marker;
pub use markers::Markers;
pub use poll::{poll, LIVE_CLIENT_URL};
pub use snapshot::Snapshot;
pub use status_line::status_line;
pub use timing_hint::TimingHint;
