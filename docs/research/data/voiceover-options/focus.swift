import AppKit
import AVFoundation

// An Accessory app that speaks, and reports whether the frontmost app changed.
let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let synth = AVSpeechSynthesizer()
let before = NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "nil"
let u = AVSpeechUtterance(string: "Ward the river before you push.")
u.voice = AVSpeechSynthesisVoice(language: "en-US")
synth.speak(u)
var samples: [String] = []
for _ in 0..<40 {
  RunLoop.current.run(until: Date().addingTimeInterval(0.1))
  samples.append(NSWorkspace.shared.frontmostApplication?.bundleIdentifier ?? "nil")
}
print("before:", before)
print("during:", Set(samples))
print("self active:", app.isActive)
