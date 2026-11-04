use super::proposed_change::ProposedChange;
use super::proposed_swap::ProposedSwap;

/// What an answer asks the overlay to show, each part `None` when it asks nothing.
#[derive(Debug, Default)]
pub(crate) struct AnswerParts {
    pub(crate) text: Option<String>,
    pub(crate) change: Option<ProposedChange>,
    pub(crate) swap: Option<ProposedSwap>,
}
