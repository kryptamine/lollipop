use super::answer::Answer;
use super::answer_parts::AnswerParts;
use super::proposed_change::ProposedChange;
use super::proposed_swap::ProposedSwap;

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
        self.into_parts().text
    }

    /// The text, and the Next item and the Swap the model proposes. A proposal with no
    /// reason to show is dropped.
    pub(crate) fn into_parts(self) -> AnswerParts {
        match self {
            Self::Answered(answer) => {
                let reason = answer.reason.filter(|reason| !is_blank(reason));
                let change = answer
                    .next_item
                    .zip(reason)
                    .map(|(item, reason)| ProposedChange { item, reason });
                let swap = answer
                    .swap
                    .filter(|swap| !is_blank(&swap.reason))
                    .map(|swap| ProposedSwap {
                        sell: swap.sell,
                        buy: swap.buy,
                        reason: swap.reason,
                    });
                AnswerParts {
                    text: answer.advice.filter(|advice| !is_blank(advice)),
                    change,
                    swap,
                }
            }
            Self::Unavailable(text) => AnswerParts {
                text: Some(text.to_string()),
                ..AnswerParts::default()
            },
            Self::Nothing => AnswerParts::default(),
        }
    }
}

fn is_blank(text: &str) -> bool {
    text.trim().is_empty()
}
