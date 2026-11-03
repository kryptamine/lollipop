# Item statistics from Riot match data

Research for "Item statistics from Riot match data" (#90), part of "Map: AI
Advice" (#86). Done 2026-11-02 against patches 16.20 and 16.21.

**Answer.** Every item fact Advice needs is already in the two documents the
collector fetches for each kept match: the timeline's `ITEM_*` events give each
purchase with a millisecond timestamp, and the match's `item0`-`item6` give the
final build. Item statistics therefore cost **zero extra Riot calls**, which keeps
the personal key's budget ([ADR 0011](../adr/0011-public-tables-on-a-personal-key.md))
as it is. Candidates per Cell are the most bought completed items, about 8 per
champion and role to cover 90% of purchases. Splitting them by the enemy team's
damage profile works, while splitting by enemy champion does not have the games.
Adding candidates grows the 3.4 MB table by about 0.6-1.1 MB.

## Sources

- Riot's match-v5 reference, <https://developer.riotgames.com/apis#match-v5>, read
  as served by `developer.riotgames.com/api-details/match-v5`: `ParticipantDto`,
  `FramesTimeLineDto`, `EventsTimeLineDto`.
- Data Dragon, <https://ddragon.leagueoflegends.com/cdn/16.21.1/data/en_US/item.json>
  and `16.20.1`, `champion.json`, and `api/versions.json`.
- The Benchmark pilot's data (`scripts/pilot/collect.mjs`, kept outside the repo
  as `.probe-logs/benchmark-pilot/`): 529 EUW1 ranked solo matches with every
  `ITEM_*` event per frame, and 6 full scrubbed match and timeline documents.
  See [benchmark-pilot.md](benchmark-pilot.md).
- The published table, `v1/benchmark-table-3.json` in the public bucket.
- `crates/collector/src/`: `match_payload.rs`, `timeline_payload.rs`,
  `kept_match.rs`, `store/schema.sql`, `table/benchmark_file.rs`.

Claims marked **[observed]** come from the pilot data, not from Riot's docs.

## What the collector has today

- **Fetches**, per kept match: `GET /lol/match/v5/matches/{id}` and
  `GET /lol/match/v5/matches/{id}/timeline`, after a match-id listing. About 3.1
  calls per kept match.
- **Reads** from the match: `gameDuration`, `gameVersion`, `queueId`, and per
  participant `participantId`, `championName`, `teamPosition`,
  `gameEndedInEarlySurrender`. From the timeline: per frame `minionsKilled`,
  `jungleMinionsKilled`, `totalGold`, `level`, and `ELITE_MONSTER_KILL` events.
  Every other event lands in `Event::Other`.
- **Stores** in SQLite: `matches`, `participants` (champion, role), `minute_values`
  and `first_kills`. No item, damage or win field. About 8 KB per match.
- **Builds** a JSON file of Cells with `minutes` only.

So the item data is downloaded on every call and thrown away at parse time. The
change is in the parse structs, the schema and the build, not in the fetching.

## What match-v5 gives

### Final build, in the match document

`ParticipantDto` documents, all `int`:

- `item0`-`item5`, the six slots at game end, and `item6`, the trinket. An empty
  slot is `0`.
- `itemsPurchased`, `consumablesPurchased`: counts only.
- `physicalDamageDealtToChampions`, `magicDamageDealtToChampions`,
  `trueDamageDealtToChampions`, `totalDamageDealtToChampions`, and the matching
  `...DamageTaken` fields.
- `win`.

**[observed]** `roleBoundItem` also appears (for example `1220`, "Unleashed
Teleport (Top Lane Quest Reward)" in Data Dragon), not listed in the reference.

### Purchases over time, in the timeline

The reference documents `EventsTimeLineDto` with three fields only: `timestamp`,
`realTimestamp` and `type`. Event types and their other fields are undocumented,
so the shapes below are **[observed]** over the 6 full timelines, and stable over
the 529 matches:

| `type` | Fields | Meaning |
| --- | --- | --- |
| `ITEM_PURCHASED` | `participantId`, `itemId`, `timestamp` | A purchase |
| `ITEM_SOLD` | `participantId`, `itemId`, `timestamp` | A sale |
| `ITEM_DESTROYED` | `participantId`, `itemId`, `timestamp` | Consumed into a build, or used up (a potion) |
| `ITEM_UNDO` | `participantId`, `beforeId`, `afterId`, `goldGain`, `timestamp` | Undo in shop |

- `realTimestamp` is absent on item events. `timestamp` is ms from game start,
  exact, not the frame's minute.
- `ITEM_UNDO` of a purchase has `beforeId` = the item, `afterId` = 0 and
  `goldGain` > 0 (47 of 54). The other 7 have `beforeId` 0 and an `afterId`, an
  undone sale or swap. A purchase count must cancel undone purchases.
- Two `ITEM_PURCHASED` of `3865` (World Atlas) per match sit at `participantId` 0,
  `timestamp` 0: the support item granted at start. Participant 0 is no player.
- Per match: median 219 `ITEM_PURCHASED`, of which about 29 completed items and
  10 boots. Half are components (50%), then consumables (21%) and starters (11%).
- **Items that transform carry no purchase event.** Rebuilding each player's
  completed items from events matches `item0`-`item5` for 58 of 60 players; the
  two misses are Seraph's Embrace (`3040`), which Archangel's Staff becomes on
  full mana. Muramana and the support item upgrades are transforms of the same
  kind, not checked here. The
  final build is the source for what a player ended with; events are the source
  for order and timing.

### Pilot figures

Over 5,290 participants, with "completed" as defined in the next section:

| | Share of players | Median minute |
| --- | --- | --- |
| First completed item | 99% | 12 |
| Second | 89% | 20 |
| Third | 61% | 26 |
| Tier-2 boots | 94% | |

A median player completes 3 items. Only the first two are near universal, so
Advice before minute 20 is about items one to three.

## Classifying items with Data Dragon

`item.json` for 16.21.1 holds 863 entries, many for other modes, so filtering is
required. The fields that do it: `maps["11"]` (Summoner's Rift), `gold.purchasable`,
`gold.total`, `tags`, `into`, `from`, `depth`.

| Class | Rule used here | Count on 16.21 |
| --- | --- | --- |
| Completed | on map 11, purchasable, no `into`, `gold.total` ≥ 2200, not `Boots`, `Consumable` or `Trinket` | 138 |
| Boots | `Boots` tag and `depth` ≥ 2 (tier 2 builds into tier 3) | 13 |
| Starter | no `from`, total ≤ 500, `Lane` or `Jungle` tag | Doran's, support and jungle items |

Every item id in the 529 matches resolved against 16.20.1 or 16.21.1. A table
build should read Data Dragon for each patch in its Patch window, since an item can
leave a patch. Data Dragon is a static CDN that needs no Riot key, so it does not
touch the rate limit. The 2200 g line is a heuristic: it drops Mejai's
Soulstealer (1500), which is completed in game terms.

## Deriving candidates per Cell

### Plain popularity

For each Cell, count the players who bought each completed item (net of undo),
and keep the most bought until they cover a share of purchases. Pooled over tiers,
for the 133 champion-role groups with at least 15 players in the pilot:

| Coverage of purchases | Items needed, median | Max |
| --- | --- | --- |
| 80% | 5 | 12 |
| 90% | 8 | 17 |
| 95% | 10 | 20 |

A group buys 13 distinct completed items at the median, 27 at most (Kai'Sa
bottom). The most common first item takes 67% of first purchases at the median.
Pooled per role, the lists are far longer (31-54 items for 90%), so the
champion is what makes a list short. The pilot has about 3 games per Cell, so
these are champion-role figures; tier likely changes the list little, which would
let items pool over tiers and keep n high. That needs checking on the full store.

Useful per candidate: share of games bought, median purchase minute, and the slot
(first, second, third). `win` is in the match document as well, but a win rate per
item per Cell needs far more games than a pick rate, so it is not a first step.

### Against enemy champions

A matchup split does not have the games. A Cell at the target n=300 sees each of
about 170 enemy champions in roughly 300 × 5 / 170 ≈ 9 games. Item counts per
matchup would be noise.

### Against the enemy damage profile

A profile in a few buckets keeps n. The signal is plain even with a crude static
profile, Data Dragon's `info.magic` > `info.attack` per enemy champion, over the
4,232 non-support players:

| Enemy AP champions | Players | Mercury's Treads | Plated Steelcaps | Any MR completed item |
| --- | --- | --- | --- | --- |
| 0 | 96 | 4% | 35% | 10% |
| 1 | 768 | 6% | 23% | 9% |
| 2 | 1,852 | 10% | 19% | 11% |
| 3 | 1,324 | 18% | 13% | 16% |
| 4+ | 192 | 24% | 7% | 21% |

A better profile is measured, and still costs no calls: `magicDamageDealtToChampions`
over `totalDamageDealtToChampions` per champion, from the match documents. That is
a small per-champion table (about 170 numbers). At runtime the app knows the enemy
champions from Live Client Data's `allPlayers`, sums their profiles, and reads the
matching bucket. Three buckets (physical-heavy, mixed, magic-heavy) put about 100
games in each at n=300, enough for a share to within about ±0.1.

## Storage in SQLite

Two new tables would hold it, both from documents already fetched:

- Completed items and boots per participant with `at_ms`, from `ITEM_PURCHASED`
  minus undone ones: about 40 rows per match.
- Final `item0`-`item5` and damage dealt to champions by type, per participant: 10
  rows per match.

At the density of `minute_values` that is under 1 KB per match, about 10% on the
8 KB measured. Storing every `ITEM_PURCHASED` would be about 4 KB, half again, for
components and potions nothing reads.

## Size added to the Benchmark table

Today's table, `benchmark-table-3.json`: 958 Cell entries over 235 champion-role
pairs, 3.4 MB as served (453 KB gzipped). R2 serves it uncompressed
(`Content-Type: application/json`, no `Content-Encoding`).

Measured by adding synthetic candidate lists to that file, each candidate
`{"item":…,"share":…,"minute":…}`:

| Per Cell entry | Raw | Gzipped |
| --- | --- | --- |
| 12 items + 3 boots | +0.6 MB (+18%) | +112 KB |
| Same, with a share for each of 3 damage-profile buckets | +1.1 MB (+33%) | +204 KB |
| 20 items + 4 boots, 3 buckets | +1.7 MB (+53%) | +307 KB |

Lists keyed by champion and role, not by tier, would be about a quarter of this
(235 lists against 958). The per-champion damage profile adds a few KB. Item
names and prices stay out: the app can map ids through Data Dragon, as it already
must for `total` (see [live-game-probe.md](live-game-probe.md)).

## Open

- Whether item choices differ by tier enough to keep candidates per tier.
- The full store's coverage curves, now that it holds 12,293+ matches.
- Which items need a rule beyond Data Dragon: transforms (Seraph's, Muramana),
  support item upgrades, and `roleBoundItem`.
- Whether candidates go in the Benchmark table or a sibling file with its own
  schema version, which is a format decision for #86.
