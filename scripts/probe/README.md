# Live-game probe

Throwaway tooling for "Probe the APIs and the overlay in a real League game on
macOS". Output lands in `.probe-logs/`, which is ignored. Player names are
replaced and the LCU password is never written, but read a log before sharing it.

Build once with `scripts/probe/build-overlay.sh`.

## 1. Practice Tool, windowed, about 15 minutes

Start `node scripts/probe/record.mjs` before opening the client, and keep the
terminal beside the game. Type a line and press Enter to log a note.

1. Last-hit 25 minions one at a time. Note the HUD CS at 1, 5, 9, 10, 11 and 25.
2. Compare gold with the HUD and note any difference.
3. Buy and sell a component, a completed item and a potion, then drink a potion.
4. Level up and note how long the terminal takes to show it.
5. Kill a drake, grubs, Herald and Baron if the tool allows it, and note the clock.

## 2. Overlay matrix, same Practice Tool game

With the recorder still running, set the display mode in the game's video
settings, then run the command below and return to the game. Each variant is
announced aloud and lasts 15 seconds. Answer the prompts afterwards.

    scripts/probe/overlay-matrix.sh fullscreen
    scripts/probe/overlay-matrix.sh borderless
    scripts/probe/overlay-matrix.sh windowed

Use only the display modes the game offers.

## 3. One full Summoner's Rift game

Keep the recorder running from champion select to the end of the game. Note:

- when Herald despawns if nobody takes it
- how cannon waves look around 14:00 and 25:00, including with an inhibitor down

A ranked game also compares `position` with the LCU's `assignedPosition`.

## 4. Other modes, as they get played

Run the recorder in blind, draft, ranked, ARAM, Arena and custom. Each game adds
`position`, `gameMode` and `mapNumber` for that mode.

## Resolve

Give the logs to an agent working the ticket. Unplayed modes are recorded as not
tested, not as empty.
