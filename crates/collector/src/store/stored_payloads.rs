use lollipop_core::game_inputs::Patch;

use super::Store;
use crate::table::StoredPayloads;

// A match kept before payloads were archived has none, and gives no Item candidates.
const STORED_PAYLOADS: &str = "SELECT m.match_id, m.patch_major, m.patch_minor, p.game, p.timeline
    FROM matches m
    JOIN match_payloads p ON p.match = m.id
    WHERE (m.patch_major, m.patch_minor) >= (?, ?)
    ORDER BY m.id";

impl Store {
    /// Hands over each archived match from `floor` on, in turn.
    pub(crate) fn for_each_stored_payloads<E: From<rusqlite::Error>>(
        &self,
        floor: Patch,
        mut visit: impl FnMut(StoredPayloads) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut statement = self.connection.prepare(STORED_PAYLOADS)?;
        let mut rows = statement.query([floor.major, floor.minor])?;
        while let Some(row) = rows.next()? {
            visit(StoredPayloads {
                match_id: row.get(0)?,
                patch: Patch {
                    major: row.get(1)?,
                    minor: row.get(2)?,
                },
                game: row.get(3)?,
                timeline: row.get(4)?,
            })?;
        }
        Ok(())
    }
}
