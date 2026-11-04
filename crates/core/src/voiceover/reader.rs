use crate::advice::Trigger;
use crate::live::ShownAdvice;

use super::speech::Speech;
use super::spoken::spoken;

/// Keeps the voice on the Advice and the Build change the player sees, reading each one
/// at most once.
#[derive(Debug, Default)]
pub struct Reader {
    read: Option<(Trigger, String)>,
    read_change: Option<String>,
    speaking: bool,
}

impl Reader {
    /// `heard` and `heard_change` are what to read: `None` with Voiceover off or the
    /// overlay hidden. One already read stays silent when it shows again after a hide,
    /// and both new at once, as from one answer, are read in one go.
    pub fn follow(
        &mut self,
        heard: Option<&ShownAdvice>,
        heard_change: Option<&str>,
    ) -> Option<Speech> {
        if heard.is_none() && heard_change.is_none() {
            return std::mem::take(&mut self.speaking).then_some(Speech::Stop);
        }
        let mut texts = Vec::new();
        if let Some(advice) = heard {
            let already_read = self
                .read
                .as_ref()
                .is_some_and(|(trigger, text)| *trigger == advice.trigger && *text == advice.text);
            if !already_read {
                self.read = Some((advice.trigger, advice.text.clone()));
                texts.push(advice.text.clone());
            }
        }
        if let Some(change) = heard_change {
            if self.read_change.as_deref() != Some(change) {
                self.read_change = Some(change.to_string());
                texts.push(change.to_string());
            }
        }
        if texts.is_empty() {
            return None;
        }
        self.speaking = true;
        Some(Speech::Read(spoken(&texts.join(" "))))
    }
}
