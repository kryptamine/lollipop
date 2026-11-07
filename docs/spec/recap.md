# Recap

After a game, Lollipop shows a Recap: how the game went against its Cell, minute
by minute, and up to three Lessons for the next one. Scope settled in
[#198](https://github.com/kryptamine/lollipop/issues/198). Terms follow
[`GLOSSARY.md`](../../GLOSSARY.md).

## When it is made

- On the poll that sees `GameEnd`, which also gives Victory or Defeat.
- Otherwise, on a failed poll while a game is held, once the League client's
  `/lol-gameflow/v1/gameflow-phase` is `WaitingForStats`, `PreEndOfGame` or
  `EndOfGame`. Live Client Data drops about 1 s after `GameEnd`, so most games
  end this way, with no result.
- Not for a game with fewer than five minutes seen, such as a remake or a game
  the app joined near its end.
- Only the last Recap is kept, in memory, until the app quits.

## What it holds

| Part | Source |
| --- | --- |
| Champion, role, length, K/D/A | The last poll that saw the game |
| CS and Gold earned a minute | The first poll of each game minute |
| Their Benchmarks | What the Marker strip shows beside its Markers at that poll |
| Deaths | Every `ChampionKill` with the player as victim, to anything |
| Slow farm | Each stretch the CS pace Marker was below |
| Lessons | OpenAI, after the rest |

A support gets no CS Benchmark, and a game first seen past its first minute no
Gold earned, as on the Marker strip.

## Lessons

- Asked once per Recap, which needs the Recap window on, the Advice Widget on,
  "Lessons after a game" on and a key the run has not seen refused. They are no
  Trigger: they never reach the Advice Widget or change the Build.
- When not asked, the Recap carries the first reason that holds, in that order, and
  the window says it.
- Their own schema: `{"lessons": [string]}`. More than three, or one over 160
  characters, drops them all.
- The input holds the game line, CS and Gold earned against their Benchmarks every
  fifth minute and the last, each death and its killer's champion, the slow farm
  stretches and all nine other players. Never a Riot ID or summoner name.
- 30 s to answer, with the Advisor's one retry. They count against the per-game
  request cap of the game that starts after the one they review.
- The window tells apart a model with no Lessons and a call that failed. A failure
  is logged as `Advice lessons:` and never spends the next game's "could not reach
  OpenAI".

## Window

[ADR 0024](../adr/0024-recap-window-opens-on-its-own.md) lets it open on its own.

- Opens once per Recap at `PreEndOfGame` or `EndOfGame`. A client that does not
  answer is not one of them, so the window waits. "Recap after a game" in the
  General tab turns this off, and the Lessons with it.
- Never made key; closing it hides the app.
- A header, a CS chart, a Gold earned chart and the Lessons. Each chart draws the
  player solid and the Benchmark dashed, deaths as a cross along the top, slow farm
  as a shaded band, with a crosshair and the values on hover.

## Out of scope

History of past Recaps and trends across games, level and K/D/A charts, and a menu
bar item to reopen the last Recap.
