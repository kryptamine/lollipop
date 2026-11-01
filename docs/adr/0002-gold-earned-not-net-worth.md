---
status: accepted
---

# Compare Gold earned, not net worth

The gold Marker compares Gold earned on both sides: `totalGold` from Match-V5 timelines against increases in `currentGold` counted live. Net worth looks like the natural metric, but no version of it can be compared accurately with the data available.

## Considered options

- **Live net worth against `totalGold`:** a shift that depends on role. Median gap at minute 15 from replayed inventories: top -100, middle -175, bottom -400, jungle -450, support -550.
- **Net worth on both sides:** item upgrades emit no timeline event. Replayed final inventories matched exactly for 12 of 60 participants, with a value error of 1600 at p95. Fixing that needs a hand-kept transformation table per patch.

## Consequences

- The app needs no Data Dragon, and net worth leaves the product.
- A count that starts late cannot be recovered, so the game shows a dash for gold.
- The live count has not been checked against Riot's `goldEarned`. If one ranked game with a sale misses by more than 50 gold, this decision reopens.
