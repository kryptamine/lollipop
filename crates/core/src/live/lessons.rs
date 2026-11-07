use serde::Serialize;

use crate::advice::LessonsOff;

/// A Recap's Lessons, which arrive after the rest of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "state", content = "detail")]
pub enum Lessons {
    /// Not asked for, and why.
    Off(LessonsOff),
    Asking,
    /// Most costly first.
    Given(Vec<String>),
    /// The model found none worth giving.
    Empty,
    /// OpenAI failed, or this game's requests were spent.
    Failed,
}
