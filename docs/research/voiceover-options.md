# Voiceover options

Research for "Voiceover options" (#89), part of "Map: AI Advice" (#86). Sources
read and probes run on 2026-11-02, on macOS 27.0.1 (Apple silicon).

The question: OpenAI TTS against macOS `AVSpeechSynthesizer` for Voiceover,
compared on latency to first audio, cost per Advice, Russian and English voice
quality, streaming, and how a Tauri Accessory app plays audio without taking
focus (ADR 0006).

## Answer

| | OpenAI Speech API (`gpt-4o-mini-tts`) | OpenAI Realtime (`gpt-realtime-2.1-mini`) | `AVSpeechSynthesizer` |
|---|---|---|---|
| Lifetime | **Shuts down 2027-01-06** | Current replacement | OS API, macOS 10.14+ |
| First audio | Not published, not measured (no key) | Not published, not measured | **14-69 ms warm, 239-620 ms cold** (measured) |
| Cost per Advice (~8 s) | ~$0.002 (derived) | ~$0.003 (derived) | Free |
| Russian | Supported, voices "optimized for English" | Same voices | Milena, compact quality only by default |
| English | 13 voices, `marin`/`cedar` best | 10 voices, `marin`/`cedar` best | Samantha compact plus novelty voices by default |
| Streaming | Chunked HTTP, `pcm`/`wav` fastest | WebSocket/WebRTC deltas | Buffer callback, or the system plays it |
| Network, key | Needs both | Needs both | Neither |
| Disclosure | Must tell the user the voice is AI | Same policy | None |

**Recommendation:** ship Voiceover on `AVSpeechSynthesizer` first, called through
`objc2-avf-audio` from `platform`. It is free, offline, starts in tens of
milliseconds, needs no audio crate, and did not take focus from an Accessory app.
Its weak point is Russian quality: the only Russian voice installed by default is
Milena at compact quality. OpenAI's dedicated TTS models shut down on 2027-01-06,
so an OpenAI voice means the Realtime API now, which is a WebSocket session and a
second billing path. Keep that as a later option, behind a measurement of its first
audio latency.

## OpenAI

### Models and their lifetime

- The Speech API (`POST /v1/audio/speech`) takes `tts-1`, `tts-1-hd` and
  `gpt-4o-mini-tts` ([API reference][speech-ref]).
- **All three shut down on 2027-01-06.** The deprecations page lists them under
  "2026-10-01: Text-to-speech models", with "Migrate to `gpt-realtime-2.1-mini`
  before the shutdown date" ([deprecations][deprecations]).
- `gpt-realtime-2.1-mini` is not a Speech API model: its model page lists only
  `v1/realtime`, over WebRTC, WebSocket or SIP ([model page][rt-mini]). The TTS
  guide says the same ([TTS guide][tts-guide]). The Speech endpoint itself has no
  deprecation entry, but no model would remain for it.
- `gpt-realtime-mini` snapshots also shut down, on 2027-01-20, with the same
  replacement ([deprecations][deprecations]).

So an OpenAI Voiceover built now on the Speech API has about two months to live.
The lasting OpenAI path is the Realtime API.

### Speech API facts (until shutdown)

From the API reference ([speech-ref]) and the guide ([tts-guide]):

- `input` up to 4,096 characters; `gpt-4o-mini-tts` caps input at 2,000 tokens
  ([model page][mini-tts]).
- `instructions` steers tone, accent and pace; it does not work with `tts-1` or
  `tts-1-hd`.
- `response_format`: `mp3` (default), `opus`, `aac`, `flac`, `wav`, `pcm`. `pcm`
  is raw 24 kHz 16-bit signed samples with no header.
- `speed` 0.25 to 4.0. `stream_format` is `audio` (default) or `sse`.
- Streaming is chunked transfer encoding, so playback can start before the file
  is complete. The guide recommends `wav` or `pcm` "for the fastest response
  times".

### Realtime API facts

From the Realtime conversations guide ([rt-guide]):

- A verbatim line is a `response.create` with an empty `input` array and
  instructions such as "Say exactly the following:". `response.conversation:
  "none"` makes it out of band, so no context builds up.
- Output audio arrives as base64 chunks in `response.output_audio.delta` events.
  PCM is `{"type": "audio/pcm", "rate": 24000}`.
- Over WebSocket the client owns playback. WebRTC is recommended for browsers.
- The voice cannot change once a session has spoken.

The model follows an instruction to say text exactly; it is not a guarantee. A
Speech API call reads its input; a Realtime response is generated. This matters for
Advice that names items, which code has already checked against the candidates.

### Voices and languages

- Speech API: 13 voices (alloy, ash, ballad, coral, echo, fable, nova, onyx, sage,
  shimmer, verse, marin, cedar); `tts-1` models take 9 of them. "For best quality,
  we recommend using `marin` or `cedar`" ([tts-guide]).
- Realtime: 10 voices, without fable, nova and onyx ([rt-guide]).
- Languages follow Whisper's list, which includes Russian. The voices are
  "currently optimized for English"; the guide says performance is good across
  the listed languages ([tts-guide]). Russian quality was not heard for this note.

### Latency

Neither the guides nor the model pages publish a time to first audio. It was not
measured here: no OpenAI key was available to the probe. It is the open number for
the Realtime path, and it adds to the Advice model's own answer time.

### Cost

Prices from the pricing page ([pricing]), per 1M tokens unless stated:

| Model | Text in | Audio out |
|---|---|---|
| `gpt-4o-mini-tts` | $0.60 | $12.00 |
| `gpt-realtime-2.1-mini` | $0.60 (text out $2.40) | $20.00 |
| `tts-1` | $15.00 per 1M characters | |
| `tts-1-hd` | $30.00 per 1M characters | |

No page states how many audio tokens one second of speech costs. Assuming 1,200
output tokens per minute (20 per second, the rate OpenAI's 2024 Realtime launch
used to turn $200 per 1M into $0.24 per minute), an Advice of about 25 words
(~150 characters, ~8 s of speech; the probe's 115-character line ran 6.4-6.8 s)
costs:

| Model | Per Advice | 20 Advice in a game |
|---|---|---|
| `gpt-4o-mini-tts` | ~$0.002 | ~$0.04 |
| `tts-1` | ~$0.002 | ~$0.05 |
| `gpt-realtime-2.1-mini` | ~$0.003, plus instruction tokens | ~$0.07 |

These are derived, not billed. Either way it is cents per game, but it is a
second line on the player's own OpenAI bill.

### Policy

The TTS guide says OpenAI's usage policies require "a clear disclosure to end
users that the TTS voice they are hearing is AI-generated and not a human voice"
([tts-guide]). An OpenAI Voiceover needs that line in the settings window.

## `AVSpeechSynthesizer`

### API

From Apple's AVFAudio documentation ([avss], [write], [voice-init], [quality]):

- `AVSpeechSynthesizer` (macOS 10.14+) speaks `AVSpeechUtterance`s from a queue,
  and can pause or stop. It must be retained until speech ends.
- `speak(_:)` plays through the system's output. `write(_:toBufferCallback:)`
  (macOS 10.15+) hands back `AVAudioPCMBuffer`s instead, for an app that plays them
  itself.
- `AVSpeechSynthesisVoice(language:)` takes a BCP 47 code, `nil` for the system's
  language, and returns `nil` when no voice matches.
- Quality is `default` ("available on the device by default"), `enhanced` and
  `premium` (macOS 13+), the last two "you must download to use".
- `usesApplicationAudioSession` is not available on macOS: there is no audio
  session to configure ([uses-session]).

Rust reaches it through `objc2-avf-audio` 0.3.2: `speakUtterance`,
`writeUtterance_toBufferCallback` and `stopSpeakingAtBoundary` behind the
`AVSpeechSynthesis` feature, the buffer callback also needing `AVAudioBuffer` and
`block2` ([objc2-avf-audio]). The app already depends on `objc2` 0.6 and `block2`
0.6.

### Voices on a default install

`AVSpeechSynthesisVoice.speechVoices()` on this Mac, which has downloaded no voices:

- `ru-RU`: only `com.apple.voice.super-compact.ru-RU.Milena`, quality `default`.
- `en-US`: `com.apple.voice.compact.en-US.Samantha`, the Eloquence voices (Eddy,
  Flo, Reed, Sandy and others) and the novelty voices (Bells, Zarvox, Whisper),
  all quality `default`.

Better Russian and English voices exist as downloads in System Settings, which a
player has to fetch. The app can list voices and pick the best installed one, but
cannot download one itself. Milena at super-compact quality is clearly robotic;
how much that matters is a listening call for the owner.

### Latency (measured)

Probe: a Swift binary calling `write(_:toBufferCallback:)` on one
synthesizer, three rounds of one English and one Russian line (115 characters each),
timing the first non-empty buffer.

| Line | First buffer | Whole line | Audio length |
|---|---|---|---|
| en, cold | 620 ms | 687 ms | 6.44 s |
| ru, cold | 239 ms | 308 ms | 6.83 s |
| en, warm | 14-15 ms | 84-85 ms | 6.44 s |
| ru, warm | 28-69 ms | 100-141 ms | 6.83 s |

Output was 22,050 Hz mono. The first utterance pays a voice load; after that a
line synthesizes about 70 times faster than it plays. Speaking a short warm-up line
at volume 0 when a game starts would hide the cold cost.

## Playing audio without taking focus

ADR 0006 holds that Lollipop never activates itself. Playing sound does not need
activation on macOS: audio goes to Core Audio, which has no notion of the frontmost
app.

**Probe:** a Swift binary set `NSApplication.setActivationPolicy(.accessory)`,
called `AVSpeechSynthesizer.speak(_:)`, and sampled
`NSWorkspace.frontmostApplication` every 100 ms for 4 s. The frontmost app stayed
the terminal for every sample, and `NSApplication.isActive` stayed false. One run,
from a binary rather than an app bundle, and not over League; the release
checklist should repeat it in a game.

The ways to play, each without a window or activation:

1. **`AVSpeechSynthesizer.speak(_:)`**: the system plays the speech. No audio code
   in Lollipop at all. Fits the native voice only.
2. **`rodio` 0.23** over `cpal` 0.18 (Core Audio on macOS): open the default
   sink with `DeviceSinkBuilder::open_default_sink()` and play through a `Player`
   ([rodio]). Its default Symphonia decoders cover mp3, wav, flac and vorbis; raw
   PCM goes through its sample buffers. This is what the OpenAI path needs, for the
   24 kHz PCM from either API. The handle must be kept alive, since playback stops
   when it drops.
3. **The webview** (`<audio>` or Web Audio in the overlay): not tested. WKWebView
   applies autoplay rules to pages with no user gesture, and the overlay never gets
   one. It would also move audio into `ui/`, which only draws state.

Nothing here ducks League's own sound; the two mix at the system volume.

## Open

- OpenAI first-audio latency on Realtime, measured with a real key.
- How Russian sounds on `marin`/`cedar` against Milena, by ear.
- The release-checklist run of the focus probe inside a game.

## Reproducing

Both probes are in `docs/research/data/voiceover-options/`. Build each with
`swiftc -O tts.swift -o tts` (or `focus.swift`) and run the binary; `focus` reports
the frontmost app, so start it from the app that should stay in front.

## Sources

Read 2026-11-02.

- OpenAI API reference, Create speech.
  <https://developers.openai.com/api/docs/api-reference/audio/createSpeech>
- OpenAI, Text to speech guide.
  <https://developers.openai.com/api/docs/guides/text-to-speech>
- OpenAI, Deprecations. <https://developers.openai.com/api/docs/deprecations>
- OpenAI, Pricing. <https://developers.openai.com/api/docs/pricing>
- OpenAI, `gpt-4o-mini-tts` model page.
  <https://developers.openai.com/api/docs/models/gpt-4o-mini-tts>
- OpenAI, `gpt-realtime-2.1-mini` model page.
  <https://developers.openai.com/api/docs/models/gpt-realtime-2.1-mini>
- OpenAI, Realtime conversations guide.
  <https://developers.openai.com/api/docs/guides/realtime-conversations>
- OpenAI, "Introducing the Realtime API" (2024), for $0.24 per output minute at
  $200 per 1M tokens. <https://openai.com/index/introducing-the-realtime-api/>
  (403 to the fetcher; the figure is as quoted elsewhere, not re-read.)
- Apple, `AVSpeechSynthesizer`.
  <https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer>
- Apple, `write(_:toBufferCallback:)`.
  <https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer/write(_:tobuffercallback:)>
- Apple, `AVSpeechSynthesisVoice.init(language:)`.
  <https://developer.apple.com/documentation/avfaudio/avspeechsynthesisvoice/init(language:)>
- Apple, `AVSpeechSynthesisVoiceQuality`.
  <https://developer.apple.com/documentation/avfaudio/avspeechsynthesisvoicequality>
- Apple, `usesApplicationAudioSession` (no macOS entry).
  <https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer/usesapplicationaudiosession>
- docs.rs, `objc2_avf_audio::AVSpeechSynthesizer` 0.3.2.
  <https://docs.rs/objc2-avf-audio/0.3.2/objc2_avf_audio/struct.AVSpeechSynthesizer.html>
- docs.rs, `rodio` 0.23.0. <https://docs.rs/rodio/0.23.0/rodio/>
- crates.io API for current versions: rodio 0.23.0, cpal 0.18.2,
  objc2-avf-audio 0.3.2.

[speech-ref]: https://developers.openai.com/api/docs/api-reference/audio/createSpeech
[tts-guide]: https://developers.openai.com/api/docs/guides/text-to-speech
[deprecations]: https://developers.openai.com/api/docs/deprecations
[pricing]: https://developers.openai.com/api/docs/pricing
[mini-tts]: https://developers.openai.com/api/docs/models/gpt-4o-mini-tts
[rt-mini]: https://developers.openai.com/api/docs/models/gpt-realtime-2.1-mini
[rt-guide]: https://developers.openai.com/api/docs/guides/realtime-conversations
[avss]: https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer
[write]: https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer/write(_:tobuffercallback:)
[voice-init]: https://developer.apple.com/documentation/avfaudio/avspeechsynthesisvoice/init(language:)
[quality]: https://developer.apple.com/documentation/avfaudio/avspeechsynthesisvoicequality
[uses-session]: https://developer.apple.com/documentation/avfaudio/avspeechsynthesizer/usesapplicationaudiosession
[objc2-avf-audio]: https://docs.rs/objc2-avf-audio/0.3.2/objc2_avf_audio/struct.AVSpeechSynthesizer.html
[rodio]: https://docs.rs/rodio/0.23.0/rodio/
