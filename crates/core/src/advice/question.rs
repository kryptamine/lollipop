use std::time::{Duration, Instant};

/// The longest one attempt of a Trigger may take, so a retry still lands in time.
pub(crate) const TRIGGER_ATTEMPT: Duration = Duration::from_secs(8);

/// What one Trigger, or a Recap, asks OpenAI.
#[derive(Debug)]
pub(crate) struct Question<'a> {
    /// What the log names the asker by, such as a Trigger's id.
    pub(crate) asker: &'static str,
    pub(crate) instructions: &'a str,
    pub(crate) input: &'a str,
    /// When the asker drops a late answer.
    pub(crate) deadline: Instant,
    /// The longest one attempt may take, within the deadline.
    pub(crate) attempt_timeout: Duration,
}
