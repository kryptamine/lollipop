use std::time::Instant;

use super::trigger::Trigger;

/// What one Trigger asks OpenAI.
#[derive(Debug)]
pub(crate) struct Question<'a> {
    pub(crate) trigger: Trigger,
    pub(crate) instructions: &'a str,
    pub(crate) input: &'a str,
    /// When the Trigger drops a late answer.
    pub(crate) deadline: Instant,
}
