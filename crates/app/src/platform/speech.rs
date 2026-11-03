use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_avf_audio::{
    AVSpeechBoundary, AVSpeechSynthesisVoice, AVSpeechSynthesizer, AVSpeechUtterance,
};
use objc2_foundation::NSString;

thread_local! {
    // SAFETY: `new` takes no arguments; the marker on every call keeps it on one thread.
    static SYNTHESIZER: Retained<AVSpeechSynthesizer> = unsafe { AVSpeechSynthesizer::new() };
}

/// Speaks `text` on the system volume once the current word of any reading ends,
/// dropping the rest of it: nothing ever queues. A `volume` of 0 speaks silently.
pub(crate) fn speak(_: MainThreadMarker, text: &str, voice: &AVSpeechSynthesisVoice, volume: f32) {
    SYNTHESIZER.with(|synthesizer| {
        // SAFETY: the utterance is built here and handed over whole, on the main thread.
        unsafe {
            let utterance = AVSpeechUtterance::speechUtteranceWithString(&NSString::from_str(text));
            utterance.setVoice(Some(voice));
            utterance.setVolume(volume);
            synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Word);
            synthesizer.speakUtterance(&utterance);
        }
    });
}

pub(crate) fn stop_speaking(_: MainThreadMarker) {
    SYNTHESIZER.with(|synthesizer| {
        // SAFETY: stopping a synthesizer that is silent does nothing.
        unsafe {
            synthesizer.stopSpeakingAtBoundary(AVSpeechBoundary::Immediate);
        }
    });
}
