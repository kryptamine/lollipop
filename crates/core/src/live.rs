//! Polls Riot's Live Client Data API for the active player's own stats.

mod client;
mod clock;
mod payload;
mod snapshot;

pub use client::{LiveClient, LIVE_CLIENT_URL};
pub use clock::{Clock, SystemClock};
pub use snapshot::Snapshot;
