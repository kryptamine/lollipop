use std::time::Instant;

/// What one Trigger, or a Recap, asks OpenAI.
#[derive(Debug)]
pub(crate) struct Question<'a> {
    /// What the log names the asker by, such as a Trigger's id.
    pub(crate) asker: &'static str,
    pub(crate) instructions: &'a str,
    pub(crate) input: &'a str,
    /// When the asker drops a late answer.
    pub(crate) deadline: Instant,
}
