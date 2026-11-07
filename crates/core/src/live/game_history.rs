use std::path::PathBuf;

use rusqlite::{params, Connection};

use super::trend_game::{TrendGame, TREND_GAMES};

/// The `SQLite` file on this machine that keeps each game's CS at 10:00: see ADR 0025.
#[derive(Debug, Clone)]
pub struct GameHistory {
    path: PathBuf,
}

impl GameHistory {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Keeps `game`, if any, and returns the Trend on `champion`, oldest first. A file
    /// that cannot be written leaves the Trend at this game alone.
    pub(super) fn trend_with(&self, champion: &str, game: Option<TrendGame>) -> Vec<TrendGame> {
        self.try_trend_with(champion, game)
            .unwrap_or_else(|_| game.into_iter().collect())
    }

    fn try_trend_with(
        &self,
        champion: &str,
        game: Option<TrendGame>,
    ) -> Result<Vec<TrendGame>, Box<dyn std::error::Error>> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(&self.path)?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS cs_at_ten (
                id INTEGER PRIMARY KEY,
                champion TEXT NOT NULL,
                cs INTEGER NOT NULL,
                benchmark REAL
            )",
        )?;
        if let Some(TrendGame { cs, benchmark }) = game {
            connection.execute(
                "INSERT INTO cs_at_ten (champion, cs, benchmark) VALUES (?1, ?2, ?3)",
                params![champion, cs, benchmark],
            )?;
        }
        let mut statement = connection.prepare(
            "SELECT cs, benchmark FROM cs_at_ten WHERE champion = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let mut games = statement
            .query_map(params![champion, TREND_GAMES], |row| {
                Ok(TrendGame {
                    cs: row.get(0)?,
                    benchmark: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        games.reverse();
        Ok(games)
    }
}
