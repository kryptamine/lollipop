# Live-game probe on macOS

Measured on 2026-11-01, patch 16.21, for "Probe the APIs and the overlay in a
real League game on macOS". It checks what the earlier research on Live Client
data, auto-detection, schedules and overlay technology could not run.

Three games: two Practice Tool games and one ranked solo game on Summoner's Rift
(Warwick, jungle, 33:15). The overlay ran only in Practice Tool.

## Findings

| Question | Result |
|---|---|
| `creepScore` floor | Floored to 10 |
| Freshness of CS, gold, items, level | No noticeable lag |
| `price` | Data Dragon's `base`, not `total` |
| `position` | `NONE` in Practice Tool, `JUNGLE` in ranked |
| LCU on macOS | Works, and stays reachable during a game |
| Patch in Live Client Data | None; LCU only |
| Events named by the ticket | All fire |
| Overlay in Fullscreen | Only above the shield level |
| Overlay in Windowed | Any tested level |

## Player data

### CS

| HUD | `creepScore` |
|---|---|
| 5 | 0 |
| 16 | 10 |
| 25 | 20 |

All 3,983 state records of the ranked game hold a multiple of 10. The largest
error is therefore 9 CS. The step to 20 arrived in the same record as the +21
gold for that minion, so the floor adds no visible delay.

### Gold

`currentGold` is a float and changes about every 0.5 s while gold accrues. It
matched the HUD when compared by eye. No exact simultaneous reading was logged.

### Items

| Item | `price` | Data Dragon `base` | Data Dragon `total` | Data Dragon `sell` | Gold on buying | Gold on selling |
|---|---|---|---|---|---|---|
| Long Sword | 350 | 350 | 350 | 245 | -349 | +245 |
| Infinity Edge | 725 | 725 | 3500 | 2450 | -3500 | +2450 |
| Health Potion | 50 | 50 | 50 | 20 | -49 | not sold |
| Stealth Ward | 0 | 0 | 0 | 0 | not bought | not sold |

The 1-gold gaps are passive income between polls. Other findings:

- Net worth built from `price` undercounts completed items. It needs `total`
  from Data Dragon by `itemID`.
- `slot` changes when items are reordered, so it does not identify an item.
- A drunk potion leaves `items` at once.
- Items and the gold change land within one 250 ms poll.

### Level

A Practice Tool level-up appeared in the terminal at once, by eye. The delay was
not measured.

## Role, rank and patch

| Mode | `gameMode` | `mapNumber` | Queue | `position` | `assignedPosition` |
|---|---|---|---|---|---|
| Practice Tool | `PRACTICETOOL` | 11 | 3140 | `NONE` | empty |
| Ranked solo | `CLASSIC` | 11 | 420 | `JUNGLE` | `jungle` |

The two role fields agree in the one ranked game, with different casing.
`position` did not change during that game.

### League client API

- The lockfile is at `/Applications/League of Legends.app/Contents/LoL/lockfile`.
- `--install-directory` from `ps` gave the same directory once `LeagueClientUx`
  was up.
- The API timed out for about 10 s after the lockfile appeared, then answered.
- It stayed at 200 through all three games.
- `/lol-gameflow/v1/gameflow-phase`, `/lol-gameflow/v1/session`,
  `/lol-champ-select/v1/session`, `/lol-summoner/v1/current-summoner`,
  `/lol-ranked/v1/current-ranked-stats` and `/lol-patch/v1/game-version` all
  answered.
- `current-ranked-stats` has `queues` entries with `queueType`, `tier`,
  `division` and `leaguePoints`.
- Phases seen: `None`, `Lobby`, `Matchmaking`, `ReadyCheck`, `ChampSelect`,
  `InProgress`, `WaitingForStats`, `EndOfGame`.

### Patch

Live Client Data has no patch field. `gameData` holds only `gameMode`, `gameTime`,
`mapName`, `mapNumber` and `mapTerrain`, and `info.version` in its OpenAPI document
is `1.0.0`. LCU `/lol-patch/v1/game-version` returned
`16.21.8342754+branch.releases-16-21.code.public.content.release.anticheat.vanguard`.

Live Client Data also lists control paths such as `/Exit`. Nothing here calls them.

## Availability of `allgamedata`

| Moment | Status |
|---|---|
| Before the game process is up | `ECONNREFUSED` |
| About 9-10 s after phase `InProgress` | 404 |
| About 2-3 s later | 200, first `gameTime` near 0 |
| At game end, right after `GameEnd` | `ECONNRESET`, then `ECONNREFUSED` |

In one load of three, 200 came about 0.25 s before the active player could be
matched in the payload. A reader must treat that as no game yet. `parse_snapshot`
already returns `None` there.

## Events

| Event | Fields besides `EventID`, `EventName`, `EventTime` | Observed |
|---|---|---|
| `GameStart` | none | At `gameTime` near 0 |
| `MinionsSpawning` | none | 30.05 s in all three games |
| `DragonKill` | `DragonType`, `KillerName`, `Assisters`, `Stolen` | Types seen: `Hextech`, `Water`, `Air`, `Earth` |
| `HordeKill` | `KillerName`, `Assisters`, `Stolen` | One per grub, three per game |
| `HeraldKill` | `KillerName`, `Assisters`, `Stolen` | |
| `BaronKill` | `KillerName`, `Assisters`, `Stolen` | |
| `TurretKilled` | `TurretKilled`, `KillerName`, `Assisters` | Example `Turret_TChaos_L1_P3_...` |
| `InhibKilled` | `InhibKilled`, `KillerName`, `Assisters` | Example `Inhib_TChaos_L1_P1_...` |
| `InhibRespawned` | `InhibRespawned` | Same name as the kill; no `KillerName` |
| `GameEnd` | `Result` | `Win` |

Also seen: `FirstBrick`, `FirstBlood`, `ChampionKill`, `Multikill`, `Ace`.

### Timing

- Inhibitor respawn: 300.2, 300.0 and 300.0 s after the kill.
- Drake: appeared 5:00 after a 5:30 kill in Practice Tool, by eye. Spawns have no
  event.
- With Fast Forward, `gameTime` jumps. An event can be logged a few seconds after
  its `EventTime`, so `EventTime` is the value to use.

### Killer names

A player reads as a player name. A minion reads as `Minion_T100L0S39N0240`, where
`T100` or `T200` gives the team. No non-champion drake kill was seen, so that
format is not confirmed for drakes.

## Overlay

Release build, hand-bundled `.app`, ad-hoc signed. Each variant ran 15 s against a
real game, with a window-list snapshot at the end. "Shield" means `setLevel` to
`CGShieldingWindowLevel() + 1`, here 2147483629, through `ns_window()`.

| Display mode | Regular, level 5 | Accessory | Accessory + `FullScreenAuxiliary` | Shield only | All three |
|---|---|---|---|---|---|
| Fullscreen | hidden | hidden | hidden | **drawn** | **drawn** |
| Windowed | drawn | not run | not run | drawn | not run |

- The game's Window Mode has no borderless option.
- League's windows are at layer 2147483628 and cover the 2560x1440 display in
  Fullscreen, and at layer 0 in Windowed. 2147483628 equals
  `CGShieldingWindowLevel()`.
- Other apps' windows stay in the on-screen list during Fullscreen. Together with
  the layer, that points to a captured display rather than a fullscreen Space.
  This is inference: `CGDisplayIsCaptured` is unavailable in current SDKs.
- Accessory is not needed, so the Dock icon can stay.
- Clicks passed through the strip to the game in the shield-only Fullscreen run
  and in Windowed.
- One shield snapshot lists the overlay with no League window, after the player
  left the game. At that level the overlay stays on screen over other apps, so
  the app must hide it when League is not frontmost.

## Not tested

- `position` in draft, blind, ARAM, Arena, custom and Swiftplay.
- Herald despawn time: Herald was killed in each game where it spawned.
- The cannon rule at 14:00 and 25:00, and while an inhibitor is down.
- A non-champion drake kill.
- The overlay in a real match, its FPS cost, and a `tauri build` bundle.
- Windows, for all of the above.

## Data

`data/live-game-probe/` holds the logs, sanitized before committing.

- `recorder.ndjson`: all three games, by `kind`: `state`, `event`, `first-payload`,
  `live-status`, `lcu-status`, `lcu-phase`, `lcu-base`, `lcu-champ-select`,
  `game-restarted`.
- `overlay-matrix.ndjson`: one record per variant, and one with the player's answer
  for a run. `seen` holds variant numbers for that run. One Fullscreen rerun has no
  answer record. The stray `т` in `issues` is a typing slip for `none`.

Sanitizing:

- Player names were already replaced on recording by `self`, `player-N` and `tag`.
- Wall-clock `at` became `t`, seconds since the recorder started.
- Removed: rank values, the list of other open apps, and item text in `state`.
  Rank key names remain.

## Reproducing

The tooling is in `scripts/probe/`; see its README. On this branch, `main.rs` takes
the variant from `LOLLIPOP_PROBE`.
