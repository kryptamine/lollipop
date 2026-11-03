import AVFoundation
import Foundation

let voices = AVSpeechSynthesisVoice.speechVoices().filter { $0.language.hasPrefix("ru") || $0.language == "en-US" }
for v in voices { print("voice", v.language, v.name, v.quality.rawValue, v.identifier) }
let synth = AVSpeechSynthesizer()

func probe(_ text: String, _ lang: String, _ tag: String) {
  let u = AVSpeechUtterance(string: text)
  u.voice = AVSpeechSynthesisVoice(language: lang)
  let t0 = Date()
  var first: Double? = nil
  var frames = 0
  var rate = 0.0
  var done = false
  synth.write(u) { buf in
    guard let pcm = buf as? AVAudioPCMBuffer else { return }
    if pcm.frameLength == 0 { done = true; return }
    if first == nil { first = Date().timeIntervalSince(t0) }
    frames += Int(pcm.frameLength)
    rate = pcm.format.sampleRate
  }
  while !done { RunLoop.current.run(until: Date().addingTimeInterval(0.005)) }
  let total = Date().timeIntervalSince(t0)
  print(String(format: "%@ first=%.0fms total=%.0fms audio=%.2fs rate=%.0f", tag, (first ?? -1) * 1000, total * 1000, Double(frames) / rate, rate))
}

let en = "You died to the jungler again. Ward the river before you push, and back with nine hundred gold for your first item."
let ru = "Ты снова погиб от лесника. Поставь вард у реки перед пушем и вернись на базу с девятьюстами золота за первый предмет."
for i in 0..<3 {
  probe(en, "en-US", "en#\(i)")
  probe(ru, "ru-RU", "ru#\(i)")
}
