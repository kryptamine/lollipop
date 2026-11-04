use super::answer::Answer;
use super::proposed_change::ProposedChange;

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
        self.into_parts().0
    }

    /// The text, and the Next item the model proposes. A proposal with no reason to
    /// show is dropped.
    pub(crate) fn into_parts(self) -> (Option<String>, Option<ProposedChange>) {
        match self {
            Self::Answered(answer) => {
                let reason = answer.reason.filter(|reason| !reason.trim().is_empty());
                let change = answer
                    .next_item
                    .zip(reason)
                    .map(|(item, reason)| ProposedChange { item, reason });
                let text = answer.advice.filter(|advice| !advice.trim().is_empty());
                (text, change)
            }
            Self::Unavailable(text) => (Some(text.to_string()), None),
            Self::Nothing => (None, None),
        }
    }
}
