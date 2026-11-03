use crate::advice::Asked;

/// What a Trigger that may name an item has to show.
pub(super) struct Answered {
    pub(super) text: String,
    pub(super) item: Option<u32>,
}

impl Answered {
    /// `None` for a null or blank `advice`, which shows nothing.
    pub(super) fn of(asked: Asked) -> Option<Self> {
        match asked {
            Asked::Answered(answer) => {
                let text = answer.advice.filter(|advice| !advice.trim().is_empty())?;
                Some(Self {
                    text,
                    item: answer.item,
                })
            }
            Asked::Unavailable(text) => Some(Self {
                text: text.to_string(),
                item: None,
            }),
            Asked::Nothing => None,
        }
    }
}
