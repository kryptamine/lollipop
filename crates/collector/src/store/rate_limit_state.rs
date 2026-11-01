use rusqlite::{params, OptionalExtension};

use super::Store;
use crate::app_rate_limit::AppRateLimit;

const APP_RATE_LIMIT: &str = "app_rate_limit";
const CALLS_HELD_UNTIL: &str = "calls_held_until";

// Kept on disk, so a restart inside an open window or a Retry-After still honours it.
impl Store {
    pub(crate) fn learn_rate_limit(&self, header: &str) -> rusqlite::Result<()> {
        self.set_state(APP_RATE_LIMIT, header)
    }

    pub(crate) fn hold_calls_until(&self, at_ms: u64) -> rusqlite::Result<()> {
        self.set_state(CALLS_HELD_UNTIL, at_ms)
    }

    /// When the next call may go out. A time at or before `now_ms` means now.
    pub(crate) fn next_call_at(&self, now_ms: u64) -> rusqlite::Result<u64> {
        let mut next = self.state(CALLS_HELD_UNTIL)?.unwrap_or(now_ms);
        for limit in self.rate_limits()? {
            let oldest_counted: Option<u64> = self
                .connection
                .query_row(
                    "SELECT at_ms FROM api_calls ORDER BY at_ms DESC LIMIT 1 OFFSET ?",
                    [limit.max_calls - 1],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(at_ms) = oldest_counted {
                next = next.max(at_ms + limit.window_ms);
            }
        }
        Ok(next)
    }

    pub(crate) fn record_call(&self, now_ms: u64) -> rusqlite::Result<()> {
        let longest_window = self
            .rate_limits()?
            .iter()
            .map(|limit| limit.window_ms)
            .max();
        if let Some(longest_window) = longest_window {
            self.connection.execute(
                "DELETE FROM api_calls WHERE at_ms <= ?",
                [now_ms.saturating_sub(longest_window)],
            )?;
        }
        self.connection
            .execute("INSERT INTO api_calls (at_ms) VALUES (?)", params![now_ms])?;
        Ok(())
    }

    fn rate_limits(&self) -> rusqlite::Result<Vec<AppRateLimit>> {
        let header: Option<String> = self.state(APP_RATE_LIMIT)?;
        Ok(header
            .and_then(|header| AppRateLimit::parse_all(&header))
            .unwrap_or_default())
    }
}
