use crate::advice::Trigger;
use crate::live::ShownAdvice;

use super::speech::Speech;

/// Keeps the voice on the Advice the player sees, reading each one at most once.
#[derive(Debug, Default)]
pub struct Reader {
    read: Option<(Trigger, String)>,
    speaking: bool,
}

impl Reader {
    /// `heard` is the Advice to read: `None` with Voiceover off or the overlay hidden.
    /// An Advice already read stays silent when it shows again after a hide.
    pub fn follow(&mut self, heard: Option<&ShownAdvice>) -> Option<Speech> {
        let Some(advice) = heard else {
            return std::mem::take(&mut self.speaking).then_some(Speech::Stop);
        };
        let already_read = self
            .read
            .as_ref()
            .is_some_and(|(trigger, text)| *trigger == advice.trigger && *text == advice.text);
        if already_read {
            return None;
        }
        self.read = Some((advice.trigger, advice.text.clone()));
        self.speaking = true;
        Some(Speech::Read(advice.text.clone()))
    }
}
