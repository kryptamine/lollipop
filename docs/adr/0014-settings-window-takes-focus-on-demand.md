---
status: accepted
---

# A settings window takes focus on demand

Advice needs settings a menu cannot hold, such as an API key and a language, so Lollipop gets a settings window. This replaces ADR 0006's "a settings window would take focus and hide the overlay, so the MVP has none". The rest of ADR 0006 holds: the app stays an Accessory app, never activates on its own, and hides the overlay by alpha.

- The window opens only from "Settings…" (Cmd-,) in the menu bar, never on its own, first launch included. A second "Settings…" raises the open window.
- Opening it activates Lollipop, so the overlay hides while it is frontmost. This is accepted, in a game too.
- Closing it hides the app, so focus returns to the app it came from and the overlay comes back when that app is League.
- It is a second Tauri window drawn by the Preact view, so no native UI code is added outside `platform`.

## Consequences

- The menu bar keeps only the status line, "Settings…" and Quit. Every switch moves to the window.
- The window has three tabs. General: launch at login. Overlay: one switch per Widget, the Advice Widget included. Advice: the OpenAI key, Advice language, one switch per Trigger, and Voiceover.
- The Advice Widget's switch is the only Advice on/off switch: with it off, nothing calls OpenAI and Voiceover is greyed out.
- A change applies as soon as it is made, in a running game too. There is no Save button.
