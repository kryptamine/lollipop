//! Walks seed players on Riot's web API and stores their matches in compact form.

#![allow(
    clippy::future_not_send,
    reason = "runs on a current-thread runtime, and a rusqlite Connection is not Sync"
)]

mod app_rate_limit;
mod clock;
mod collect;
mod collect_error;
mod config;
mod kept_match;
mod kept_participant;
mod league_entries;
mod match_payload;
mod minute_values;
mod riot_api;
mod seed_tier;
mod seed_walk;
mod store;
mod timeline_payload;

pub use clock::{Clock, SystemClock};
pub use collect::collect;
pub use collect_error::CollectError;
pub use config::Config;
pub use seed_tier::SeedTier;
