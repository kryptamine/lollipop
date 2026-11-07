use serde::Serialize;

/// Why a Recap's Lessons were not asked for, which the Recap window says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LessonsOff {
    NoKey,
    /// OpenAI refused the saved key this run.
    KeyRefused,
    AdviceWidgetOff,
    SwitchedOff,
    /// Nobody would see them: the Recap window is off.
    RecapOff,
}
