use objc2::rc::Retained;
use objc2::runtime::NSObjectProtocol;
use objc2::sel;
use objc2_avf_audio::{AVSpeechSynthesisVoice, AVSpeechSynthesisVoiceTraits};
use objc2_foundation::NSString;

/// The best installed voice for the language of `tag`, such as `ru-RU`: premium over
/// enhanced over default, then `tag`'s region, then the system's pick for it.
pub(super) fn best_voice(tag: &str) -> Option<Retained<AVSpeechSynthesisVoice>> {
    let primary = primary_subtag(tag);
    // SAFETY: class methods and property getters with no arguments to get wrong.
    unsafe {
        let system_pick = AVSpeechSynthesisVoice::voiceWithLanguage(Some(&NSString::from_str(tag)))
            .map(|voice| voice.identifier().to_string());
        AVSpeechSynthesisVoice::speechVoices()
            .to_vec()
            .into_iter()
            .filter(|voice| primary_subtag(&voice.language().to_string()) == primary)
            .filter(|voice| !is_character_voice(voice))
            .max_by_key(|voice| {
                let identifier = Some(voice.identifier().to_string());
                (
                    voice.quality().0,
                    voice.language().to_string() == tag,
                    identifier == system_pick,
                )
            })
    }
}

pub(crate) fn voice_installed(tag: &str) -> bool {
    best_voice(tag).is_some()
}

fn primary_subtag(tag: &str) -> &str {
    tag.split(['-', '_']).next().unwrap_or(tag)
}

// Novelty voices such as Bubbles are installed as default-quality English voices.
// `voiceTraits` exists from macOS 14 only.
fn is_character_voice(voice: &AVSpeechSynthesisVoice) -> bool {
    if !voice.respondsToSelector(sel!(voiceTraits)) {
        return false;
    }
    // SAFETY: the selector was just checked to exist on this macOS.
    let traits = unsafe { voice.voiceTraits() };
    traits.intersects(
        AVSpeechSynthesisVoiceTraits::IsNoveltyVoice
            | AVSpeechSynthesisVoiceTraits::IsPersonalVoice,
    )
}
