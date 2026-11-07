---
status: accepted
---

# The Trend is kept in a local SQLite file

A Recap tells one game, and a player who wants to see progress needs the same number across games. So the app keeps each game's CS at 10:00, with the Cell's CS Benchmark at that minute, in `history.sqlite` in its data directory, and every Recap carries the last ten games on its champion. This amends [ADR 0003](0003-benchmarks-are-static-tables-from-our-own-collector.md): the app still has no backend, but it now keeps a database of its own.

- Nothing leaves the machine and there is no account. Deleting the file clears the Trend.
- The Benchmark is stored with each game rather than read again later, since the tier, the role and the Benchmark table can change between games.
- A game is keyed by the champion's Match-V5 id from `rawChampionName`, so the client's language does not split one champion's games. A payload without it keeps no game.
- CS at 10:00 is read from the first poll within 5 s past 10:00. A game shorter, joined later or lagging past that is not kept. A file that cannot be written costs the Trend, never the Recap.

## Considered options

- **A JSON file:** simpler for ten numbers, but the history only grows, and more metrics or minutes are the likely next step. The collector already builds SQLite in.
- **Riot's Match-V5 history:** needs the player's Riot ID and a key the app does not ship ([ADR 0008](0008-public-and-local-riot-apis-only.md)).

## Consequences

- Games played before this release have no Trend.
- A table rebuilt with different Cells shifts the Benchmark between games; the Trend shows each game against its own.
