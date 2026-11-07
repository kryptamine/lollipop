use serde::Serialize;

use super::lessons::Lessons;
use super::outcome::Outcome;
use super::recap_minute::RecapMinute;
use super::scores::Scores;
use super::stretch::Stretch;
use crate::game_inputs::Role;

/// How one game went against its Cell, minute by minute, made once it ends.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recap {
    /// In the client's language.
    pub champion: String,
    pub role: Option<Role>,
    /// `None` when no poll saw `GameEnd`, the only event that carries it.
    pub outcome: Option<Outcome>,
    /// Game time at the end, in seconds.
    pub length: f64,
    pub scores: Option<Scores>,
    /// From the first poll the app saw, so a game joined late starts late.
    pub minutes: Vec<RecapMinute>,
    /// Game time of each death of the player, to anything.
    pub deaths: Vec<f64>,
    /// Where the CS pace Marker was below.
    pub slow_farm: Vec<Stretch>,
    pub lessons: Lessons,
}
