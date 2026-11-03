use super::answer::Answer;

/// What asking came to, as the Trigger shows it.
#[derive(Debug)]
pub(crate) enum Asked {
    Answered(Answer),
    /// Shown in place of the Trigger's Advice, to say why there is none.
    Unavailable(&'static str),
    Nothing,
}
