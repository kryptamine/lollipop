use std::fmt;

use super::advice_call::AdviceCall;
use super::openai::OpenAi;
use super::trigger::Trigger;

type CallFor = Box<dyn Fn(Trigger) -> Option<AdviceCall> + Send + Sync>;

/// Where Advice comes from. `call_for` is asked as each Trigger fires, so a switch
/// flipped or a key saved mid-game applies to the next one.
pub struct Advisor {
    openai: OpenAi,
    call_for: CallFor,
}

impl Advisor {
    pub fn new(
        base_url: &str,
        call_for: impl Fn(Trigger) -> Option<AdviceCall> + Send + Sync + 'static,
    ) -> Self {
        Self {
            openai: OpenAi::new(base_url),
            call_for: Box::new(call_for),
        }
    }

    pub(crate) fn call_for(&self, trigger: Trigger) -> Option<AdviceCall> {
        (self.call_for)(trigger)
    }

    pub(crate) fn openai(&self) -> OpenAi {
        self.openai.clone()
    }
}

impl fmt::Debug for Advisor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Advisor")
            .field("openai", &self.openai)
            .finish_non_exhaustive()
    }
}
