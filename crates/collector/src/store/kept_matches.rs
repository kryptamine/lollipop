use lollipop_core::game_inputs::Patch;
use rusqlite::{params, OptionalExtension, Transaction};

use super::Store;
use crate::kept_match::KeptMatch;

// The Patch window is the two newest patches among the stored matches.
const WINDOW_FLOOR: &str = "SELECT patch_major, patch_minor FROM (
    SELECT DISTINCT patch_major, patch_minor FROM matches
    ORDER BY patch_major DESC, patch_minor DESC LIMIT 1 OFFSET 1
)";

impl Store {
    /// The oldest patch of a Patch window that holds two. Below it a match is too old.
    pub(crate) fn patch_window_floor(&self) -> rusqlite::Result<Option<Patch>> {
        self.connection
            .query_row(WINDOW_FLOOR, [], |row| {
                Ok(Patch {
                    major: row.get(0)?,
                    minor: row.get(1)?,
                })
            })
            .optional()
    }

    /// Stores a match and takes it off the queue as one step, so a kill leaves either
    /// both done or neither. A first match on a new patch shifts the Patch window.
    pub(crate) fn keep(&mut self, kept: &KeptMatch) -> rusqlite::Result<()> {
        let transaction = self.connection.transaction()?;
        insert(&transaction, kept)?;
        transaction.execute(
            "DELETE FROM pending_matches WHERE match_id = ?",
            [&kept.match_id],
        )?;
        transaction.execute(
            &format!("DELETE FROM matches WHERE (patch_major, patch_minor) < ({WINDOW_FLOOR})"),
            [],
        )?;
        transaction.commit()
    }
}

fn insert(transaction: &Transaction<'_>, kept: &KeptMatch) -> rusqlite::Result<()> {
    transaction.execute(
        "INSERT INTO matches (match_id, patch_major, patch_minor, tier) VALUES (?, ?, ?, ?)",
        params![kept.match_id, kept.patch.major, kept.patch.minor, kept.tier],
    )?;
    let id = transaction.last_insert_rowid();
    let mut participant = transaction.prepare(
        "INSERT OR REPLACE INTO participants (match, participant, champion, role)
         VALUES (?, ?, ?, ?)",
    )?;
    for row in &kept.participants {
        participant.execute(params![id, row.id, row.champion, row.role])?;
    }
    let mut minute = transaction.prepare(
        "INSERT OR REPLACE INTO minute_values (match, participant, at_ms, cs, total_gold, level)
         VALUES (?, ?, ?, ?, ?, ?)",
    )?;
    for row in &kept.minute_values {
        minute.execute(params![
            id,
            row.participant,
            row.at_ms,
            row.cs,
            row.total_gold,
            row.level
        ])?;
    }
    let mut kill = transaction
        .prepare("INSERT INTO first_kills (match, objective, at_ms) VALUES (?, ?, ?)")?;
    for (objective, at_ms) in &kept.first_kills {
        kill.execute(params![id, objective, at_ms])?;
    }
    transaction.execute(
        "INSERT INTO match_payloads (match, game, timeline) VALUES (?, ?, ?)",
        params![id, kept.archived.game, kept.archived.timeline],
    )?;
    Ok(())
}
