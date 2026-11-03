//! The `SQLite` database, which is also everything a restarted run resumes from.

mod first_kills;
mod kept_matches;
mod match_queue;
mod played_games;
mod rate_limit_state;
mod stored_payloads;

use std::path::Path;

use rusqlite::types::FromSql;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension, ToSql};

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

    /// Refuses a missing file, rather than creating an empty database to build from.
    pub(crate) fn open_read_only(path: &Path) -> rusqlite::Result<Self> {
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
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
