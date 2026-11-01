# Benchmark pilot

Results for "Run a Benchmark pilot on a few thousand matches" (#15). Collected
2026-11-01 on EUW1 with a development key, patches 16.20 and 16.21.

**Sample: 529 matches, not a few thousand.** The owner cut the run: format and
limits do not need volume, and the full check moves to the first real collection.
That is 60 matches per seed tier (Gold 109). The spread figures below are rough
estimates for that reason.

Reproduce with `scripts/pilot/collect.mjs` and `scripts/pilot/analyze.mjs`. Full
aggregates: `docs/research/data/benchmark-pilot/analysis.json`. Raw data stays out
of the repo, and PUUIDs and player names were never written to disk.

## Timeline fields

Checked over 5,290 participants and 160,040 participant frames.

| Question | Result |
| --- | --- |
| Final `totalGold` vs `goldEarned` | Equal for all 5,290 |
| `totalGold` at frame 0 | 500 for all, so `goldEarned` includes starting gold |
| `totalGold` falling between frames | Never, so it is cumulative |
| `currentGold` above `totalGold` | Never |
| Final CS vs `totalMinionsKilled` + `neutralMinionsKilled` | Equal for all |
| Final `level` vs `champLevel` | Equal for all |
| `currentGold` at frame 0 | 100 to 500: purchases at the start already show |

`currentGold` reads as unspent gold, with sale refunds added outside `goldSpent`:
`currentGold - (totalGold - goldSpent)`, at game end, has a median of -9 and a
range of -124 to +2 for players who sold nothing. For players who sold something
the median is +171, and the maximum +17,867, which I did not explain.

### Frames

- `frameInterval` is 60000 in all 529 matches, and no gap between frames is
  irregular.
- Frame timestamps run late, and the lag grows: median 16 ms at frame 1, 325 ms at
  frame 15, 663 ms at frame 35. Interpolating to the minute handles this, and at
  under 1 s the effect is minor.
- The final frame is a partial one, 0.3-60 s after the frame before it (median
  30 s), and within 1 s of `gameDuration`. It holds the final values.

## Filtering games

**`endOfGameResult` does not remove early games.** All 520 games fetched in the
resumed run read `GameComplete`, including the 12 flagged
`gameEndedInEarlySurrender`. This corrects "What are the Benchmark dimensions and
pipeline?", which relied on it.

`gameEndedInEarlySurrender` is not enough either: 4 of the 529 kept games lasted
under 15 minutes (3.9, 7.4, 7.4 and 12.7). A minimum duration
is needed as well. These games barely touch the minutes that matter, since 99.2%
of kept games reach minute 15.

Share of games reaching a minute: 15: 99%, 20: 91%, 25: 78%, 30: 47%, 35: 20%.
So 100 games at minute 30 takes about 215 games in the Cell.

## Spread within a Cell

Pooled over Cells with at least 5 games, because the sample averages 3 per Cell.
It leans to popular picks. Up to 3 games share a seed player, so the sample is
less independent than its count suggests.

Coefficient of variation at minute 15, against the estimates in the Match-V5
research (0.25 for CS, 0.15 for gold):

| Role | CS | Gold | Level |
| --- | --- | --- | --- |
| Top | 0.19 | 0.18 | 0.09 |
| Jungle | 0.14 | 0.14 | 0.07 |
| Middle | 0.16 | 0.15 | 0.07 |
| Bottom | 0.16 | 0.17 | 0.07 |
| Support | 0.34 | 0.14 | 0.09 |

### Do minimum 100 and target 300 hold?

Matches needed for a median within a relative error (minute 15):

| Metric | ±5% | ±3% |
| --- | --- | --- |
| CS, except support | 48-90 | 131-248 |
| CS, support | 280 | 776 |
| Gold | 47-82 | 129-227 |
| Level | 12-19 | 32-53 |

They hold as relative errors: 100 gives about ±5%, and 300 gives about ±3%.
Support CS misses only because its mean is low: at n=100 its error is ±1.7 CS.

### Do the starting neutral zones hold?

Absolute 95% error of a Cell's median, excluding support where it is smaller:

| Minute | CS, n=100 | CS, n=300 | Gold, n=100 | Gold, n=300 |
| --- | --- | --- | --- | --- |
| 10 | 2-3 | 1-2 | 116-144 | 67-83 |
| 15 | 4-5 | 2-3 | 205-248 | 118-143 |
| 20 | 5-6 | 3 | 290-340 | 167-196 |
| 30 | 7-9 | 4-5 | 425-562 | 246-325 |

- **±150 net worth is too narrow.** From about minute 15 at n=100, and minute 20
  at n=300, the Benchmark's own error is larger than the zone, so noise alone can
  set the Marker. A fixed zone cannot fit, since spread grows with the minute.
- **±8 CS holds until about minute 30** at n=100.
- **±1 level holds throughout.** The error is at most 0.3.

Share of games inside the zone, against a leave-one-out Cell median:

| Metric, zone | Minute 10 | Minute 15 | Minute 20 | Minute 30 |
| --- | --- | --- | --- | --- |
| CS ±8, except support | 42-60% | 25-43% | 23-34% | 18-22% |
| CS ±8, support | 84% | 73% | 65% | 42% |
| Gold ±150 | 16-23% | 9-18% | 9-14% | 3-9% |
| Level ±1 | 92-97% | 75-91% | 72-87% | 61-75% |

So the gold Marker is almost never neutral, and the level Marker rarely fires. The
30 s hold was not tested. Whether this is acceptable belongs to "How do Markers stay
accurate within the data limits?".

## Seed tier vs a lookup per participant

54 matches, 486 other participants, all ranked in solo.

- Same tier as the seed: 54.5%. Within one tier: 96.9%.
- The errors lean one way: 180 are above the seed's tier, 41 below.
- Same-tier share by seed tier runs from 35% (Iron, Platinum) to 65% (Emerald).
  Master+ reads 100% because it merges three tiers.

Shift in a Benchmark from using the seed's label, estimated from role and tier
medians, so it is noisy:

| Metric | Minute | Mean signed | Mean absolute | p95 absolute |
| --- | --- | --- | --- | --- |
| CS | 15 | -1.1 | 1.9 | 8.5 |
| CS | 20 | -1.5 | 2.7 | 12.0 |
| Gold | 15 | -25 | 62 | 285 |
| Gold | 20 | -32 | 89 | 445 |
| Level | 15 | -0.04 | 0.06 | 0.49 |

**Keep the seed's label in the private phase.** The mean shift is well inside the
zones, and tier differences are small next to the spread within a Cell. A lookup
per participant costs 10 more calls per match. Lookups also return the current
rank, not the rank at match time, so they are not ground truth. Recheck before
public tables, which a production key could afford.

Sanity check: median CS at minute 15 climbs with tier (bottom: 85 in Iron, 122 in
Master+). Support CS stays flat at about 20.

## Limits on the key

Development key, from the response headers.

| Scope | Limit |
| --- | --- |
| Application | 20/1 s, 100/2 min |
| `league-v4` entries | 50/10 s |
| `league-v4` challenger, grandmaster, master leagues | 30/10 s, 500/10 min |
| `league-v4` entries by PUUID | 20,000/10 s, 1,200,000/10 min |
| `match-v5` ids, match, timeline | 2,000/10 s each |

Only the application limit binds. A personal key may report different method
limits, so check on the first run.

Nine 429s in 1,741 calls of the resumed run: eight were `server` type (Riot's
shared service limit, a 3 s wait), and one `application`, probably from restarting with
the earlier run's window still open. A collector should persist or wait out that
window on restart.

### Throughput

The run cost about 3.1 calls per kept match, excluding rank lookups, against the
2.4 assumed. That includes 17% of fetched matches rejected for an old patch, which
a tighter time window would reduce. At 0.83 req/s this gives **about 23,000 matches
a day, not 30,000**, so every collection time in the research runs about 30% longer.

## Cells

The sample has 422 distinct champion-role pairs against the 230 assumed, because of
rare off-role games. Only 0.2% of participants have a `teamPosition` that disagrees
with `individualPosition`, and 1 of 5,290 has none.

## Not measured

- CV inside a single Cell: no Cell has more than 25 games.
- Real thin-Cell shares, since sampling per tier was equal rather than proportional.
- Whether 300 matches hold at volume, including effects from repeated players.
- The net worth comparison with live `currentGold` plus item prices.
- Regions other than EUW1.
