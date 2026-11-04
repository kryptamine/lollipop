use super::answer::Answer;

/// What asking came to, as the Trigger shows it.
#[derive(Debug)]
pub(crate) enum Asked {
    Answered(Answer),
    /// Shown in place of the Trigger's Advice, to say why there is none.
    Unavailable(&'static str),
    Nothing,
}

impl Asked {
    /// The text the Widget shows, `None` for a null or blank `advice`.
    pub(crate) fn into_text(self) -> Option<String> {
        match self {
            Self::Answered(answer) => answer.advice.filter(|advice| !advice.trim().is_empty()),
            Self::Unavailable(text) => Some(text.to_string()),
            Self::Nothing => None,
        }
    }
}
