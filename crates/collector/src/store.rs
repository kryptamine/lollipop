//! The `SQLite` database, which is also everything a restarted run resumes from.

mod kept_matches;
mod match_queue;
mod rate_limit_state;

use std::path::Path;

use rusqlite::types::FromSql;
use rusqlite::{params, Connection, OptionalExtension, ToSql};

pub(crate) use match_queue::PendingMatch;

const LAST_TIER: &str = "last_tier";

pub(crate) struct Store {
    connection: Connection,
}

impl Store {
    pub(crate) fn open(path: &Path) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        // WAL lets a table build read while the collector writes.
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", true)?;
        connection.execute_batch(include_str!("store/schema.sql"))?;
        Ok(Self { connection })
    }

    pub(crate) fn last_tier(&self) -> rusqlite::Result<Option<String>> {
        self.state(LAST_TIER)
    }

    pub(crate) fn set_last_tier(&self, tier: &str) -> rusqlite::Result<()> {
        self.set_state(LAST_TIER, tier)
    }

    fn state<T: FromSql>(&self, key: &str) -> rusqlite::Result<Option<T>> {
        self.connection
            .query_row(
                "SELECT value FROM collector_state WHERE key = ?",
                [key],
                |row| row.get(0),
            )
            .optional()
    }

    fn set_state(&self, key: &str, value: impl ToSql) -> rusqlite::Result<()> {
        self.connection.execute(
            "INSERT OR REPLACE INTO collector_state (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }
}
