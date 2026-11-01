//! Polls Riot's Live Client Data API for the active player's own stats.

mod clock;
mod gold_earned;
mod payload;
mod poll;
mod snapshot;
mod status_line;

pub use clock::{Clock, SystemClock};
pub use poll::{poll, LIVE_CLIENT_URL};
pub use snapshot::Snapshot;
pub use status_line::status_line;
