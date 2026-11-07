---
status: accepted
---

# The Recap window opens on its own after a game

A Recap is worth most right after the game, before the next queue, and a player who has to remember a menu item rarely opens it. So the Recap window opens on its own, which [ADR 0014](0014-settings-window-takes-focus-on-demand.md) ruled out for the settings window. The rest of ADR 0014 and [ADR 0006](0006-overlay-never-takes-focus.md) holds: Lollipop never activates on its own.

- It opens once per Recap, when the League client's gameflow phase reaches `PreEndOfGame` or `EndOfGame`. The game's own window has closed by then, so nothing the player is doing is covered. A client that does not answer is no post-game screen, so the window waits.
- It is ordered in with `orderFrontRegardless` and is never made key, so focus stays with the League client. A click into it activates Lollipop, as the settings window does.
- Closing it hides the app, so focus returns to the client.
- "Recap after a game" in the General tab turns it off, and with it the Lessons, which nobody would see.

The game is over when a poll sees `GameEnd`, or, since Live Client Data drops about 1 s after it, when the gameflow phase is `WaitingForStats`, `PreEndOfGame` or `EndOfGame`. The probe recorded `GameEnd` but not these phases: they are the client's documented values, to confirm in a real match.

## Consequences

- A window can appear over the client unasked. The switch is on by default, and the first release that ships it is checked in a real match.
- No poll that saw `GameEnd` means no Victory or Defeat on the Recap, since only that event carries the result.
- With no League client to ask, a game whose `GameEnd` no poll saw gets no Recap, and a Recap that was made never opens.
