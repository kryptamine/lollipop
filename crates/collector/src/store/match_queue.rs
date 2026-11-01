use rusqlite::{params, OptionalExtension};

use super::Store;

/// A match id waiting for its calls. Ids carry no PUUID, so unlike a seed it can be kept.
pub(crate) struct PendingMatch {
    pub(crate) match_id: String,
    pub(crate) tier: String,
}

impl Store {
    pub(crate) fn enqueue(&self, tier: &str, match_ids: Vec<String>) -> rusqlite::Result<()> {
        for match_id in match_ids {
            self.connection.execute(
                "INSERT OR IGNORE INTO pending_matches (match_id, tier) VALUES (?, ?)",
                params![match_id, tier],
            )?;
        }
        Ok(())
    }

    pub(crate) fn next_pending(&self) -> rusqlite::Result<Option<PendingMatch>> {
        self.connection
            .query_row(
                "SELECT match_id, tier FROM pending_matches ORDER BY id LIMIT 1",
                [],
                |row| {
                    Ok(PendingMatch {
                        match_id: row.get(0)?,
                        tier: row.get(1)?,
                    })
                },
            )
            .optional()
    }

    pub(crate) fn is_visited(&self, match_id: &str) -> rusqlite::Result<bool> {
        self.connection.query_row(
            "SELECT EXISTS (SELECT 1 FROM matches WHERE match_id = ?1)
                 OR EXISTS (SELECT 1 FROM dropped_matches WHERE match_id = ?1)
                 OR EXISTS (SELECT 1 FROM pending_matches WHERE match_id = ?1)",
            [match_id],
            |row| row.get(0),
        )
    }

    pub(crate) fn drop_match(&mut self, match_id: &str, now_ms: u64) -> rusqlite::Result<()> {
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM pending_matches WHERE match_id = ?", [match_id])?;
        transaction.execute(
            "INSERT OR REPLACE INTO dropped_matches (match_id, at_ms) VALUES (?, ?)",
            params![match_id, now_ms],
        )?;
        transaction.commit()
    }

    pub(crate) fn forget_dropped_before(&self, at_ms: u64) -> rusqlite::Result<()> {
        self.connection
            .execute("DELETE FROM dropped_matches WHERE at_ms < ?", [at_ms])?;
        Ok(())
    }
}
