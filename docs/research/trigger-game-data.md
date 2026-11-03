# What game data each Trigger can read

Answers "What game data each Trigger can read" (#87, map #86). Written
2026-11-02 from Riot's developer docs, Riot's published Live Client Data samples,
the community Riot API schema, and the recordings of the live-game probe
([live-game-probe.md](live-game-probe.md), patch 16.21, macOS). ADR 0008 limits
the sources to Live Client Data, the League client API (LCU) and the Riot web API.

Tags: **[docs]** Riot's documentation or samples, **[probe]** seen in
`data/live-game-probe/recorder.ndjson`, **[code]** what Lollipop reads today,
**[unverified]** community knowledge or inference, not checked.

## Answer

| Trigger | Signal | Data at that moment | Not reachable |
|---|---|---|---|
| Loading screen | `allgamedata` answers 200 while `GameStart` is missing | All champions, teams, positions, keystones and rune trees, summoner spells; the active player's full runes and base stats; LCU rank, queue, patch | Enemy ranks without extra LCU calls, anything about the game itself |
| Death | `ChampionKill` with `VictimName` = the active player, or `isDead` flipping true | Killer, assisters, time, `respawnTimer`, every player's items, level and KDA | Any damage breakdown: no death recap in any local or live API |
| Back to base | None. Inferred from a purchase: items change and `currentGold` drops | Same as death, plus the purchase itself | A recall that buys nothing, and the player's position |
| Periodic | Any poll, on `gameTime` | Everything above plus all events so far and the active player's live stats and gold | Other players' gold, cooldowns, positions, vision |

## Sources and transport

- Live Client Data is served by the game process at `https://127.0.0.1:2999`
  with a self-signed certificate, "during an active game". Endpoints:
  `allgamedata`, `activeplayer`, `activeplayername`, `activeplayerabilities`,
  `activeplayerrunes`, `playerlist`, `playerscores`, `playersummonerspells`,
  `playermainrunes`, `playeritems` (by `riotId`), `eventdata`, `gamestats`. **[docs]**
- No polling interval or rate limit is documented. **[docs]** Lollipop polls
  `allgamedata` once a second with an 800 ms timeout. **[code]** The probe polled
  it every 250 ms for three games without errors. **[probe]**
- The LCU is not officially supported: "no guarantees of full documentation,
  service uptime, or change communication". **[docs]** It stays reachable through a
  game on macOS. **[probe]**
- The Riot web API needs a key. ADR 0011 keeps secrets out of the app, so
  Spectator-V5 and Match-V5 are out of reach of the app; Match-V5 is post-game anyway.
- The Replay API on the same port works only in replays, and is off by default
  (`EnableReplayApi=1`). **[docs]** It gives nothing in a live game.

## Loading screen

`allgamedata` answers during the loading screen. **[probe]**

| Moment, ranked solo | Status |
|---|---|
| Gameflow phase `InProgress` | loading screen starts |
| About 9-10 s later | 404 |
| About 2-3 s later | 200, `gameTime` near 0, gold 0, no `GameStart` |
| About 56 s after the first 200 | `GameStart` |

In Practice Tool the 200 came under a second before `GameStart`, so the window
depends on load times. The first 200 can also come about 0.25 s before the active
player appears in `allPlayers`. **[probe]** A loading-screen Trigger is
"200 with the active player and no `GameStart`" and has about 50 s in ranked.

Fields in the first payload of the ranked game **[probe]**:

- `activePlayer`: `fullRunes` (`keystone`, `primaryRuneTree`, `secondaryRuneTree`,
  `generalRunes` with ids, `statRunes` with ids), `championStats` at level 1,
  `abilities` (names and ids, all at level 0), `level`, `currentGold` 0, `riotId`.
- Each player: `championName`, `rawChampionName`, `team` (`ORDER`/`CHAOS`),
  `position` (`JUNGLE` in ranked, `NONE` in Practice Tool), `runes` (keystone and
  the two trees only, with ids), `summonerSpells` (display and raw names, no id),
  `skinID`, `isBot`, `isDead`, `respawnTimer`, `items` (empty), `scores` (all 0),
  `level`, `riotId`.
- `gameData`: `gameMode`, `gameTime`, `mapName`, `mapNumber`, `mapTerrain`. No patch.
- `events`: no `GameStart` yet.

The recorder kept only the active player's entry when sanitizing, so that all
ten players are present at that moment is **[unverified]**; Riot's sample and every
later state carry ten. Other players' `generalRunes` and `statRunes` are not
exposed: `runes` holds the keystone and trees only. **[docs]**

LCU at the loading screen:

- `/lol-ranked/v1/current-ranked-stats`: the player's own `queues` with `tier`,
  `division`, `leaguePoints`. **[probe]**
- `/lol-gameflow/v1/session`: `gameData.queue.id` (420 ranked solo). **[code]**
  **[probe]** That it also lists both teams with champion ids and spells is
  **[unverified]**.
- `/lol-patch/v1/game-version`: the patch. **[probe]**
- `/lol-champ-select/v1/session` was recorded only during `ChampSelect`. Whether
  it still answers after champ select ends is **[unverified]**; community reports
  say it returns 404 once the phase leaves `ChampSelect`. Lollipop reads it at
  `GameStart` as a role fallback, so that fallback may never succeed in a real game.
  `position` in Live Client Data covered the one ranked game.
- Other players' ranks would need their PUUIDs and an LCU ranked lookup per
  player. **[unverified]**: neither endpoint was probed, and Live Client Data
  gives Riot IDs, not PUUIDs.

## Death

`ChampionKill` carries `EventID`, `EventName`, `EventTime`, `KillerName`,
`VictimName` and `Assisters` (a list of names), and nothing else. **[docs]**
**[probe]** Nearby events add context: `Multikill` (`KillStreak`), `FirstBlood`
(`Recipient`), `Ace` (`Acer`, `AcingTeam`). **[docs]**

- Latency: the poll saw `ChampionKill` 0.02-0.27 s after its `EventTime`
  (median 0.15 s, 90 kills, 250 ms polls). **[probe]**
- `isDead` and `respawnTimer` per player give the death and the time to respawn
  without matching names. **[docs]** Their flip timing was not recorded.
- The killer's champion, items, level and scores come from `allPlayers` by name.
  `KillerName` is "a player name" and the name field it matches is not documented;
  Lollipop matches Riot ID, game name or summoner name. **[code]**
- A death to a turret, minion or monster: all 90 recorded kills had a player
  killer. Whether such a death logs a `ChampionKill`, and what `KillerName` holds,
  is **[unverified]**. `isDead` covers it either way.
- Death recap: none. No Live Client Data endpoint carries damage taken **[docs]**,
  and no LCU endpoint for it is known **[unverified]**. Match-V5's timeline has
  `victimDamageReceived` per `CHAMPION_KILL` (physical, magic, true per source
  and spell) **[docs, community schema]**, but only after the game and only with a key.
  Damage can at most be guessed from the killer's champion, items and level.
- The active player's `championStats` give health and resources at each poll, so
  health falling over the last seconds before death is readable at poll
  resolution. Nothing says what dealt it.

## Back to base

There is no recall, fountain or shop event: the documented list is `GameStart`,
`MinionsSpawning`, `FirstBrick`, `TurretKilled`, `InhibKilled`, `DragonKill`,
`HeraldKill`, `BaronKill`, `ChampionKill`, `Multikill`, `Ace` **[docs]**, and the
probe saw those plus `HordeKill`, `InhibRespawned`, `FirstBlood` and `GameEnd`.
**[probe]** Live Client Data has no position.

Inference:

- A purchase: `items` changes and `currentGold` drops by the price. Both land in
  the same 250 ms poll. **[probe]** On Summoner's Rift buying needs the shop,
  that is the fountain or being dead. **[unverified]** as a rule: item undo,
  consumables used, trinket swaps and champion exceptions need filtering.
- In the ranked game, purchases of more than 100 gold came in 16 polls, which
  cluster into about 12 visits (for example 391/392 s, 627/630 s, 1355/1359 s).
  **[probe]** A few seconds of debounce turns them into one Trigger per visit.
- A purchase while `isDead` is a death shop, not a back to base.
- A recall that buys nothing leaves no mark except health and resources refilling
  fast in `championStats`. **[unverified]**: not recorded.
- Latency: the purchase is visible within one poll. The recall itself, about 8 s
  earlier, cannot be seen.

## Periodic

Any poll on `gameTime` works; `gameTime` jumps with Fast Forward in Practice Tool.
**[probe]**

- Every player: champion, level, items (`itemID`, `count`, `slot`, `price` as
  Data Dragon `base`), `scores` (`kills`, `deaths`, `assists`, `creepScore`
  floored to 10, `wardScore`), `isDead`, `respawnTimer`, `position`. **[docs]**
  **[probe]**
- The active player only: `currentGold` (float, about every 0.5 s),
  `championStats`, `abilities` with levels, `fullRunes`. **[docs]** **[probe]**
- `events`: the whole game's list on every `allgamedata` response, with rising
  `EventID`s. **[probe]** Lollipop already derives objective timings from them.
- Not exposed: other players' gold, cooldowns of spells and summoner spells,
  positions, vision, wards placed. **[docs]** Whether other players' items show
  when out of vision is **[unverified]**; the in-game scoreboard shows them.

## Polling constraints

- One `allgamedata` poll a second gives a Trigger at most about 1 s late. A 250 ms
  poll caught deaths and purchases within one poll. **[probe]**
- `allgamedata` returns every event every time, so a late-game body grows; the
  narrower endpoints (`eventdata`, `activeplayer`) exist when that matters. **[docs]**
- The API disappears at `GameEnd` (`ECONNRESET`, then `ECONNREFUSED`). **[probe]**
- The LCU timed out for about 10 s after its lockfile appeared, then answered.
  **[probe]**

## Open checks for a later probe

- All ten players present in the loading-screen payload.
- `/lol-champ-select/v1/session` and `/lol-gameflow/v1/session` contents after
  `ChampSelect`.
- A death to a turret or minion: event and `KillerName`.
- `isDead`/`respawnTimer` flip timing against the `ChampionKill`.
- A recall without a purchase in `championStats`.

## References

- Riot developer docs, Game Client API:
  <https://developer.riotgames.com/docs/lol#game-client-api>
- Riot's Live Client Data samples:
  <https://static.developer.riotgames.com/docs/lol/liveclientdata_sample.json>,
  <https://static.developer.riotgames.com/docs/lol/liveclientdata_events.json>
- Community Riot API schema (Match-V5 `MatchTimelineVictimDamage`):
  <https://www.mingweisamuel.com/riotapi-schema/openapi-3.0.0.json>
- Lollipop: `crates/core/src/live/payload.rs`, `crates/core/src/live/poll.rs`,
  `crates/core/src/game_inputs/`
