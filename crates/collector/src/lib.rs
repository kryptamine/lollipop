//! Walks seed players on Riot's web API, stores their matches in compact form, and
//! aggregates those into the Benchmark table file it publishes.

#![allow(
    clippy::future_not_send,
    reason = "runs on a current-thread runtime, and a rusqlite Connection is not Sync"
)]

mod app_rate_limit;
mod archived_payloads;
mod clock;
mod collect;
mod collect_error;
mod config;
mod frame_scores;
mod kept_match;
mod kept_participant;
mod league_entries;
mod match_payload;
mod minute_values;
mod publish;
mod riot_api;
mod schedule_check;
mod seed_tier;
mod seed_walk;
mod store;
mod table;
mod timeline_payload;

pub use clock::{Clock, SystemClock};
pub use collect::collect;
pub use collect_error::CollectError;
pub use config::Config;
pub use publish::{
    manifest_public_key, publish, rollback, GhError, PublishConfig, PublishError, Published,
    RcloneError, RollbackConfig,
};
pub use schedule_check::{check_schedule, ScheduleCheck};
pub use seed_tier::SeedTier;
pub use table::{
    build_table, DriftReport, TableConfig, TableError, DATA_DRAGON_URL, MIN_CANDIDATE_GAMES,
};
