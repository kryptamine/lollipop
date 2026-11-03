use std::fmt;

use super::language::Language;

/// What one Trigger may call OpenAI with: given only while the Advice Widget and the
/// Trigger are on and a key is saved.
#[derive(Clone)]
pub struct AdviceCall {
    pub key: String,
    pub language: Language,
}

// By hand, so that no log line can print the key.
impl fmt::Debug for AdviceCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AdviceCall")
            .field("language", &self.language)
            .finish_non_exhaustive()
    }
}
