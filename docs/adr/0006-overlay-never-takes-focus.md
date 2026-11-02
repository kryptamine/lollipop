---
status: superseded in part by ADR 0014
---

# The overlay never takes focus

Lollipop runs as an Accessory app that never activates itself, and it hides by setting alpha rather than ordering the window out. The overlay shows only while the game is frontmost, so an app that takes focus hides its own overlay and pulls the player out of League.

- `activate_ignoring_other_apps(false)`: launch left the previous app frontmost in 6 of 6 launches, against Lollipop in 6 of 6 without it. Accessory alone did not help.
- `ActivationPolicy::Accessory`: without it Lollipop sits first in Cmd-Tab. This replaces the probe's "Accessory is not needed", which only checked drawing over the game.
- `setAlphaValue` took 0-1 ms on the main thread, against 3-21 ms for `orderOut` and 2-46 ms for Tauri `hide()`.

## Consequences

- There is no Dock icon or Cmd-Tab entry, so the menu bar is the only way in, and Quit lives there.
- A settings window would take focus and hide the overlay, so the MVP has none.
- The window stays in the window list at alpha 0, so the FPS check has to measure it.
