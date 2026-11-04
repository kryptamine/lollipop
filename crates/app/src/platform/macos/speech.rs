use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_avf_audio::{AVSpeechBoundary, AVSpeechSynthesizer, AVSpeechUtterance};
use objc2_foundation::NSString;

use super::best_voice::best_voice;

thread_local! {
    // SAFETY: `new` takes no arguments; the marker on every call keeps it on one thread.
    static SYNTHESIZER: Retained<AVSpeechSynthesizer> = unsafe { AVSpeechSynthesizer::new() };
}

/// Speaks `text` in the best voice for `tag` once the current word of any reading ends,
/// dropping the rest of it: nothing ever queues. A `volume` of 0 speaks silently. Off
/// the main thread, or with no voice for `tag`, it says nothing.
pub(crate) fn speak(text: &str, tag: &str, volume: f32) {
    let (Some(_), Some(voice)) = (MainThreadMarker::new(), best_voice(tag)) else {
        return;
    };
    SYNTHESIZER.with(|synthesizer| {
        // SAFETY: the utterance is built here and handed over whole, on the main thread.
        unsafe {
            let utterance = AVSpeechUtterance::speechUtteranceWithString(&NSString::from_str(text));
            utterance.setVoice(Some(&voice));
            utterance.setVolume(volume);
            synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Word);
            synthesizer.speakUtterance(&utterance);
        }
    });
}

/// Off the main thread it does nothing.
pub(crate) fn stop_speaking() {
    if MainThreadMarker::new().is_none() {
        return;
    }
    SYNTHESIZER.with(|synthesizer| {
        // SAFETY: stopping a synthesizer that is silent does nothing.
        unsafe {
            synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
        }
    });
}
