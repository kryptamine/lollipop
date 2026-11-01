use lollipop_core::game_inputs::Patch;
use rusqlite::Row;

use super::Store;
use crate::minute_values::MinuteValues;
use crate::table::PlayedGame;

// Ordered so a build holds one champion and role in memory, not a window of matches: the
// `participants_by_champion` index gives this order with no sort spilling to disk. Riot
// leaves `teamPosition` empty on a game with no role, and no Cell has that.
const PLAYED_FRAMES: &str = "SELECT p.champion, p.role, m.tier, m.patch_major, m.patch_minor,
        p.match, v.participant, v.at_ms, v.cs, v.total_gold, v.level
    FROM participants p
    JOIN matches m ON m.id = p.match
    JOIN minute_values v ON v.match = p.match AND v.participant = p.participant
    WHERE p.role IN ('TOP', 'JUNGLE', 'MIDDLE', 'BOTTOM', 'UTILITY')
    ORDER BY p.champion, p.role, p.match, p.participant, v.at_ms";

const STORED_PATCHES: &str = "SELECT DISTINCT patch_major, patch_minor FROM matches
    ORDER BY patch_major, patch_minor";

impl Store {
    /// The patches of the stored matches, oldest first. They are the Patch window, as
    /// `keep` deletes whatever falls out of it.
    pub(crate) fn stored_patches(&self) -> rusqlite::Result<Vec<Patch>> {
        let mut statement = self.connection.prepare(STORED_PATCHES)?;
        let patches = statement.query_map([], |row| {
            Ok(Patch {
                major: row.get(0)?,
                minor: row.get(1)?,
            })
        })?;
        patches.collect()
    }

    /// Hands over each participant's game in turn, with one champion and role's together.
    pub(crate) fn for_each_played_game<E: From<rusqlite::Error>>(
        &self,
        mut visit: impl FnMut(PlayedGame) -> Result<(), E>,
    ) -> Result<(), E> {
        let mut statement = self.connection.prepare(PLAYED_FRAMES)?;
        let mut rows = statement.query([])?;
        let mut current: Option<(i64, PlayedGame)> = None;
        while let Some(row) = rows.next()? {
            let match_id: i64 = row.get(5)?;
            let frame = frame_of(row)?;
            match &mut current {
                Some((id, game)) if *id == match_id && same_participant(game, &frame) => {
                    game.frames.push(frame);
                }
                _ => {
                    let started = (match_id, game_of(row, frame)?);
                    if let Some((_, finished)) = current.replace(started) {
                        visit(finished)?;
                    }
                }
            }
        }
        current.map_or(Ok(()), |(_, game)| visit(game))
    }
}

fn same_participant(game: &PlayedGame, frame: &MinuteValues) -> bool {
    game.frames
        .first()
        .is_some_and(|first| first.participant == frame.participant)
}

fn game_of(row: &Row<'_>, frame: MinuteValues) -> rusqlite::Result<PlayedGame> {
    Ok(PlayedGame {
        champion: row.get(0)?,
        role: row.get(1)?,
        tier: row.get(2)?,
        patch: Patch {
            major: row.get(3)?,
            minor: row.get(4)?,
        },
        frames: vec![frame],
    })
}

fn frame_of(row: &Row<'_>) -> rusqlite::Result<MinuteValues> {
    Ok(MinuteValues {
        participant: row.get(6)?,
        at_ms: row.get(7)?,
        cs: row.get(8)?,
        total_gold: row.get(9)?,
        level: row.get(10)?,
    })
}
