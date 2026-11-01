// Prints how League and the overlay sit on screen, as one JSON line.
// Other apps' windows vanishing hints at a fullscreen Space; a raised League layer hints at a captured display.
import CoreGraphics
import Foundation

let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
let windows = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] ?? []

func owner(_ window: [String: Any]) -> String { window[kCGWindowOwnerName as String] as? String ?? "" }
func layer(_ window: [String: Any]) -> Int { window[kCGWindowLayer as String] as? Int ?? 0 }

let tracked = windows.filter {
    let name = owner($0).lowercased()
    return name.contains("league") || name.contains("lollipop")
}.map { window -> [String: Any] in
    let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
    return ["owner": owner(window), "layer": layer(window), "bounds": bounds]
}

let mainDisplay = CGMainDisplayID()
let output: [String: Any] = [
    "tracked": tracked,
    "layer0Owners": Array(Set(windows.filter { layer($0) == 0 }.map(owner))).sorted(),
    "shieldLevel": Int(CGShieldingWindowLevel()),
    "displaySize": [CGDisplayPixelsWide(mainDisplay), CGDisplayPixelsHigh(mainDisplay)],
]
let json = try JSONSerialization.data(withJSONObject: output, options: [.sortedKeys])
print(String(decoding: json, as: UTF8.self))
