use lollipop_core::game_inputs::Patch;

use super::Store;
use crate::schedule_check::FirstKill;

const FIRST_KILLS: &str = "SELECT m.patch_major, m.patch_minor, k.objective, MIN(k.at_ms)
    FROM first_kills k
    JOIN matches m ON m.id = k.match
    GROUP BY m.patch_major, m.patch_minor, k.objective
    ORDER BY m.patch_major, m.patch_minor, k.objective";

impl Store {
    pub(crate) fn first_kills(&self) -> rusqlite::Result<Vec<FirstKill>> {
        let mut statement = self.connection.prepare(FIRST_KILLS)?;
        let kills = statement.query_map([], |row| {
            Ok(FirstKill {
                patch: Patch {
                    major: row.get(0)?,
                    minor: row.get(1)?,
                },
                objective: row.get(2)?,
                at_ms: row.get(3)?,
            })
        })?;
        kills.collect()
    }
}
