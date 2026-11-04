# Lollipop MVP core

The MVP shows the active player green and red Markers on CS, Gold earned and
level against a per-minute Benchmark, plus Timing hints for objectives. It is
built for the owner on macOS first, then for other players. Terms follow
[`GLOSSARY.md`](../../GLOSSARY.md). Decisions that need their reasoning kept are
in [`docs/adr/`](../adr/).

Accuracy is the product: a missing Marker or Timing hint is better than a wrong
one ([ADR 0004](../adr/0004-no-marker-beats-a-wrong-marker.md)).

**Season** in this document means all patches that share a major `gameVersion`.
Patch notes number 16.21 as 26.21, so every comparison uses `gameVersion`.

## Game inputs

| Input | Source | Read |
| --- | --- | --- |
| CS, `currentGold`, level, items, champion, `position` | Live Client Data | Poll every 1 s |
| Game time, events | Live Client Data | Poll every 1 s |
| Queue, patch, tier | League client API (LCU), through the lockfile | Once, at game start |

- Lockfile: `/Applications/League of Legends.app/Contents/LoL/lockfile`.
- Queue: `gameData.queue.id` of `/lol-gameflow/v1/session`.
- Patch: `/lol-patch/v1/game-version`.
- Tier: `/lol-ranked/v1/current-ranked-stats`, the entry whose `queueType` is
  `RANKED_SOLO_5x5`. A `tier` of empty, `NONE` or `UNRANKED` is an unranked player.
  A failed request, no such entry or any other value is an unknown tier.
- Role: `position`, then LCU `assignedPosition` if it is empty or `NONE`, from the
  `myTeam` entry of `/lol-champ-select/v1/session` for `localPlayerCellId`.
- The LCU is unavailable when the lockfile cannot be read or none of queue, patch
  and ranked stats answers.
- Queue, patch, tier, role and champion hold for the whole game. A failed poll does
  not end a game: a loading screen, `GameEnd` or a clock turned back does.
- A `200` from Live Client Data does not mean the game has started. It can come
  about 56 s before `GameStart`, and once came with no active player.
- The game is loading until `GameStart` appears in `events.Events`, and has
  ended at `GameEnd`. At game end the connection resets and is then refused.

## Overlay window

One transparent click-through window covers the display. Every Widget lives in
it, placed by CSS, at level `CGShieldingWindowLevel() + 1`.

The overlay is shown only when the game is frontmost and the match is running:

| Situation | Overlay |
| --- | --- |
| Client lobby, no game | Hidden |
| Loading screen | Hidden |
| Game frontmost, match running | Shown |
| Another app, the client or Lollipop frontmost | Hidden |
| After `GameEnd` | Hidden |

- Frontmost is bundle id `com.riotgames.LeagueofLegends.GameClient`, read from
  `userInfo` of `NSWorkspaceDidActivateApplicationNotification`. There is no
  debounce.
- Hiding sets alpha to 0, and the window is ordered in once.
- `ActivationPolicy::Accessory` and `activate_ignoring_other_apps(false)` are
  both required ([ADR 0006](../adr/0006-overlay-never-takes-focus.md)).

## Marker strip

A table at the top left, with a row each for K/D/A, CS, Gold earned and level and
the columns Stat, Marker, Current and Benchmark, such as `Gold earned ▼ 2000 2450`.
The gold label reads "Gold earned", so the value cannot pass for current gold or
net worth. A neutral Marker draws nothing: the Benchmark beside it already
says the metric is compared. Its slot stays, so a state change never shifts the
layout. With no Benchmark the Marker and the Benchmark are hidden and the value
stays.

| Metric | Shown value | Compared value | Benchmark | Neutral zone |
| --- | --- | --- | --- | --- |
| CS | API value, floored to 10 | API value + 5 | Median | ±8% of Benchmark, at least ±8 |
| Gold | Gold earned, counted live | Same | Median of `totalGold` | ±8% of Benchmark, at least ±150 |
| Level | API value | Same | Mean | ±1 |
| K/D/A | API values, as `3 / 1 / 4` | Kills + assists − deaths | Mean of each, shown whole | ±2 |

- A Marker turns to a triangle on the first poll past the Neutral zone, and
  keeps it until the value is back within half of the zone or past the other edge.
- The shown Benchmark is the one at the last whole minute, rounded to a whole
  number, so it changes once a minute and a player on pace is at or above it.
  The Marker compares against the Benchmark interpolated to the current game
  time.
- Benchmarks are read by linear interpolation between minutes.
- A minute with n<100 has no Marker.
- A support keeps the CS value and has no CS Marker.
- K/D/A counts only games kept with scores, as `scores_n`, and a minute where that
  is below 100 has no K/D/A Marker. A payload without scores has no K/D/A row.
  The ±2 zone is untested against real spread.

### Counting Gold earned

See [ADR 0002](../adr/0002-gold-earned-not-net-worth.md).

- Add every increase in `currentGold`, except in a poll where an item left the
  list, which covers sales and undos.
- If the first snapshot arrives before 60 s of game time, start from 500.
- Otherwise the game shows a dash and no gold Marker. CS and level are
  unaffected. The count is not persisted, so a restart mid-game behaves the same.

### When Markers apply

Markers need all of the following, read at game start:

- **Queue:** ranked solo, ranked flex, draft or blind. Anything else shows no
  Markers and no notice.
- **Champion, role and tier:** automatic only, never set by hand or guessed.
- **A table that serves the patch:** see [Stale table](#stale-table).
- **A Cell:** the player's Cell, else adjacent tiers (±1), else no Marker. The build
  pools the adjacent tiers into the Cell's entry, so the app reads only that entry.

## Timing hints

A centre banner near the top of the screen shows the event name and a countdown,
such as `Baron 0:15`. A hint appears 45 s before a spawn and disappears at the
spawn. Hints are silent, stack with the soonest on top and grow downward. The
lead is the same for every objective and role.
Baron and Elder may carry an Elixir icon: see
[AI Advice](ai-advice.md#elixir).

Hints apply in ranked solo, ranked flex, draft, blind and Practice Tool. An
unknown queue or an unknown patch means no hints.

### Objective schedule

Compiled into the app, in one module
([ADR 0005](../adr/0005-objective-schedule-compiled-into-the-app.md)). Values
as of patch 16.21:

| Objective | First spawn | Respawn | Event |
| --- | --- | --- | --- |
| Elemental drake | 5:00 | 5:00 after the kill | `DragonKill` |
| Elder Dragon | 6:00 after a team's 4th drake | 6:00 after the kill | `DragonKill` with `DragonType: "Elder"` |
| Voidgrubs | 8:00 | None | None |
| Rift Herald | 15:00 | None | None |
| Baron Nashor | 20:00 | 6:00 after the kill | `BaronKill` |

- A team's drake count comes from `KillerName`, joined to `/playerlist`. A minion
  killer reads `Minion_T100...`, where `T100` or `T200` gives the team.
- If a `DragonKill` cannot be given a team, the drake and Elder hints stop for
  the rest of the game, because the Elder rule can no longer be applied.
- Each schedule entry has a `from_patch`, and the patch read at game start picks
  the entry, so a fix can ship before its patch. A hotfix that keeps the patch
  number applies once installed.
- Within a Season, hints run on any patch. On a new major `gameVersion` there are
  no hints until the app has an entry for that Season.

### Noticing a change

- The owner reads each patch's notes and ships a fix before the patch.
- Each collector build compares the earliest `ELITE_MONSTER_KILL` per objective
  and patch with the schedule, and prints the result. In `publish`, a kill before
  the spawn opens an issue through `gh`, once per objective and patch, even if that
  issue has since closed. `build-table` stays offline. Elder is compared with its
  earliest possible spawn, 26:00. A delay of up to a week is accepted.
- This catches a spawn moved earlier. Moving it later needs a threshold that is not
  set.
- A kill's `timestamp` is in game milliseconds. Its `monsterType` is `DRAGON`,
  `HORDE`, `RIFTHERALD` or `BARON_NASHOR`, and only a dragon has a `monsterSubType`,
  with Elder as `ELDER_DRAGON`. Seen on third-party captures of real timelines, not
  yet on one of ours.

## Notices

A notice is a muted line in the strip for the first 10 s of a game. Each
condition shows its own line. With the strip switched off, notices are not shown
and the menu's status line carries the reason. The strip's panel grows downward for
them, and its metrics do not move.

| Condition | Line | Effect |
| --- | --- | --- |
| Stale table, naming the game's patch | `Benchmarks out of date (patch 16.22)` | No Markers |
| No table, or one that does not read | `No Benchmarks yet` | No Markers |
| Unknown patch | `Patch unknown` | No Markers, no hints |
| Unknown role | `Role unknown` | No Markers |
| Unknown tier | `Tier unknown` | No Markers |
| Unranked player | `Unranked` | No Markers |
| New Season with no schedule | `No schedule for Season 17` | No hints |
| Thin Cell | None | No Marker |
| Unavailable LCU | `League client unavailable`, alone | No Markers, no hints |

An unavailable LCU means no queue, patch or tier, so it removes both Markers and
hints. If only the tier cannot be read, hints keep working. It is the one notice
with no known queue, since it is what cost the queue. Practice Tool has hints and
no Markers, so it shows only `Patch unknown` and the schedule line.

## Menu bar

The menu bar menu holds no switches: they live in the settings window it opens
([`ai-advice.md`](ai-advice.md#settings-window),
[ADR 0014](../adr/0014-settings-window-takes-focus-on-demand.md)). There is no
global hotkey.

- **Status line:** at the top, what was detected, such as
  `Master Yi · Mid · Gold`, with a reason in place of a missing value:
  `Role unknown`, `League client unavailable`, `Tier unknown`, `Unranked`, and
  `Queue unknown` or `Patch unknown` added at the end, then the table's and the
  schedule's notice lines, all game.
- **Settings…** (Cmd-, while the menu is open): opens the settings window, or
  raises it.
- **Quit.**

The settings window's General tab holds **Launch at login**, off by default. Its
Overlay tab holds one switch per Widget, on by default, including Widgets added
later.

A change applies at once, including during a game. Markers keep being computed
while the strip is off. Settings are kept in a local file, `settings.json` in the
app's config directory. With every Widget off the app keeps running with no warning.

## Benchmark table

Built offline and shipped as one versioned file
([ADR 0003](../adr/0003-benchmarks-are-static-tables-from-our-own-collector.md)).

- **Cell:** champion, role and tier. Eight tiers with Master and above merged,
  regions pooled. Matchup, map side and stored percentiles are not included.
- **File:** JSON with `schema_version`, `patch_window` and `cells`, written by
  `lollipop-collector build-table`. Schema 2 adds `damage_profile` and
  `item_candidates`, with `item_names` and `data_dragon_version`, described in the
  [AI Advice spec](ai-advice.md#in-the-benchmark-table).
- **Contents per minute:** median CS, median `totalGold`, mean level and n. Timeline
  frames are aligned to the minute by linear interpolation, and a game counts only
  towards the minutes it reached.
- **Optional per minute:** mean kills, deaths and assists, and `scores_n`, the games
  kept with scores that they come from. Left out where no game has scores. Adding
  them kept `schema_version` at 1, as the app ignores fields it does not read.
- **Thin Cells:** the source (Cell or adjacent tiers) is chosen once per Cell at
  minute 20, so a Benchmark does not jump when its source changes. Adjacent tiers
  are pooled with the Cell's own games, and a Cell with no source has no entry. There
  is no all-tier step.
- **Thresholds:** minimum n=100, target n=300. Choosing a source reads the minimum,
  and the target is what collection aims for.
- **Optional field:** `max_patch`, given to the build as `--max-patch` and left out of
  the file without it. A game patch above it makes the table stale.

### Stale table

- A table serves any patch in its Patch window, the last two patches, plus the
  patch right after the newest one, including across a Season (16.x to 17.1).
- Two patches without a new table make it a Stale table.
- The patch is read once, at game start, and an unknown patch hides Markers.

### Collection

A Rust CLI, the `collector` crate, runs continuously on a small VPS with SQLite.

- **Source:** ranked solo (queue 420), on one routing value in the private phase.
- **Filter:** drop games with `gameEndedInEarlySurrender` or under 15 minutes.
  `endOfGameResult` does not remove them.
- **Tier label:** the seed player's tier applies to all 10 participants.
- **Private phase:** collects only the owner's tier and its neighbours, for about
  three times the games per Cell for the same calls.
- **Stored per match:** per-participant minute values with kills, deaths and assists
  counted from `CHAMPION_KILL` events, the first kill time per objective, and an archive of the Match-V5 bodies with player identity and
  per-frame stats removed ([ADR 0015](../adr/0015-collector-archives-match-payloads.md)).
  PUUIDs are never stored.
- **Throughput:** about 3.1 calls per kept match, about 23,000 matches a day, and
  about 8 KB per match without the archive and 34 KB more with it, measured on the
  VPS. That is about 5 GB for a four-week window at full rate, and about 27 GB
  with the archive.
- **Retention:** raw matches outside the Patch window are deleted when it shifts.
  The collector shifts it at the first kept match of a new patch, taking the window
  as the two newest patches stored. The first build after that patch would shift it
  to the same two anyway. A match below the window is dropped before its timeline call.
- **Limits:** the application rate limit from `x-app-rate-limit`, held at 90%, plus
  `Retry-After` on a 429. That is about 21,000 matches a day against the throughput
  above. The calls made and any `Retry-After` are kept in SQLite, so a restart honours
  them.
- **Resume:** match ids wait in a queue in SQLite. A restart empties it before reading
  new seeds, and takes the tier after the last one walked. Seeds stay in memory, as
  they are PUUIDs. A rejected key, or an outage while reading seeds, ends the run for
  the service manager to restart. A single match that keeps failing is dropped.
- **Backup:** a daily copy to object storage, kept 2 weeks, which a new host restores
  before its first collection. About 1 GB
  compressed at that size.

### Build and publish

- **Cadence:** a weekly cron plus a manual run, both `lollipop-collector publish` on
  the VPS through the systemd units in `deploy/systemd/`. The window shifts at the first build
  after a patch ships, using `gameVersion` from collected matches: a build takes the
  two newest patches stored.
- **Publish gate:** the old table stays if Cells drop by more than 20%, excluding
  a `max_patch` reset and rework entries, or if the file does not read. The old table
  is the one the manifest names. A dropped Cell is one in the old table and not in the
  new, so gained Cells do not offset it. A `max_patch` reset is data past the old
  table's `max_patch`, which skips the Cell check. A rework entry excuses a champion
  while its first valid patch is above the old table's oldest patch. A bucket whose
  manifest or live table does not read also stops the publish, until a rollback.
- **`max_patch` across builds:** a build with no `--max-patch` keeps the old table's
  until stored data passes it.
- **Failure notice:** a failed build or refused publish opens an issue through
  `gh`.
- **Reworks:** `crates/collector/reworks.json`, one entry per champion and first
  valid patch, added and built before the patch ships. Older data for that champion
  is dropped, and its Marker returns at n≥100. A file that does not read fails the
  build.
- **Drift report:** each build prints champions whose median shifted sharply between
  the two patches: by more than the Neutral zone in CS or gold at minute 20, in a Cell
  with n≥100 on both. It never excludes anyone.
- **Rollback:** the manifest is the truth and the app follows it, even to an
  older version. The last 4 tables are kept. `lollipop-collector rollback --version N`
  points the manifest at a kept table, and the next publish still takes a new version.
- **Unchanged build:** a table identical to the live one is not published, so it takes
  no slot among the 4.

### Delivery to the app

- **Manifest:** `v2/manifest.json` holds the table version, its sha256 and an Ed25519
  signature over both and the schema, next to the table. The path carries the schema
  version. `v2/` is a prefix in the public bucket, beside `v2/benchmark-table-<version>.json`,
  and the version is an integer that only grows. The manifest is cached for 60 s, and a
  table for a year, as its file never changes.
- **Checks:** at launch and after every game. Downloads run in the background and
  swap atomically, and the previous file stays until the new one succeeds. A file is
  current when its sha256 is the manifest's, so a lower version is followed like any
  other. A manifest must carry a valid signature, and a download must match that sha256
  and read as a table. A failed check waits for the next one.
- **During a game:** never swap, even when Markers are hidden. A finished download waits
  until no game is held, and a failed poll does not release it. The app reads the table
  once, at game start, from `benchmark-table.json` in its data directory.
- **No table and no network:** no Markers, everything else works.
- **Hosting:** the R2 bucket `lollipop-assets`, read publicly with no token. It runs on a
  personal key: see [ADR 0011](../adr/0011-public-tables-on-a-personal-key.md).

A schema change publishes in parallel only once other people have installs. A
retired schema gets no new tables, so that app ends up with a Stale table.

## Install and update

See [ADR 0007](../adr/0007-two-release-phases.md).

| | Private phase (owner only) | Public phase (anyone else) |
| --- | --- | --- |
| Signature | Ad-hoc | Developer ID, notarized; ad-hoc until the Apple account exists ([ADR 0012](../adr/0012-ad-hoc-releases-until-the-apple-account-exists.md)) |
| Package | Local DMG | Universal DMG on `lollipop-releases` |
| Update | `git pull` and rebuild | Tauri updater |
| Build | Local | GitHub Actions on a tag |

The updater checks at launch and after each game, never during one, and downloads
in the background. A downloaded update installs with a relaunch after `GameEnd`
if no new game is running. There is no minimum version: a too-old app ends up
with a Stale table. Debug builds never update themselves.

A release starts once from `scripts/release-setup.sh`, which makes the updater
key, the Apple credentials and the GitHub secrets. After that, release-please
keeps a release pull request open on `main`, with the next version from the
Conventional Commits and its `CHANGELOG.md`. Merging it tags the version and runs
`.github/workflows/release.yml`, which leaves a draft on `lollipop-releases`. The real-match checklist runs on that draft's DMG,
and the draft is published by hand: the updater sees only published releases.

Before a public release: Riot registration and product audit, a production key,
and a recheck of the seed's tier label against a lookup per participant.

## Stack and tests

[ADR 0001](../adr/0001-tauri-preact-rust-core-stack.md) holds the stack, workspace
and linters. Probe code (`apply_probe_variant`, `LOLLIPOP_PROBE`, `scripts/probe`)
is removed in the implementation.

Tests are integration tests through an external boundary, with no unit tests:
`core` against a local HTTP server replaying recorded games and short synthetic
scenarios on a controllable clock, the built app through WebdriverIO, and a manual
checklist in a real match before release.

## First-build checks

These were not tested and can reopen a decision.

- **Gold earned:** on one ranked game with a sale, the live total is within 50
  gold of Riot's `goldEarned`. A failure reopens
  [ADR 0002](../adr/0002-gold-earned-not-net-worth.md).
- **FPS:** an average drop above 3% or noticeable frame drops first shrinks the
  window to the Widgets' area, then reopens the stack.
- **Overlay:** the full-display window in a real match, the loading and `GameEnd`
  rows, a second display, contrast and placement.
- **Menu bar:** reachable over League in Fullscreen, and whether opening it takes
  focus.
- **Settings window:** opened over League in Fullscreen, closing it hands focus
  back to League and the overlay returns. The overlay window opts out of the app
  being hidden; if it still disappears, closing reopens ADR 0014.
- **Detection:** role and tier in a real ranked game, including a lane swap and an
  unranked player, and `position` in draft and blind. Also whether
  `/lol-champ-select/v1/session` still answers at game start: the probe read it only
  in phase `ChampSelect`. If it does not, an empty `position` gives an unknown role.
  The unranked `tier` values are from memory, as the probe sanitized rank values.
- **Champion names:** a Cell is found with the client in another language and for a
  renamed champion such as Wukong. `rawChampionName` was seen once, for Master Yi.
- **Timing hints:** the 45 s lead in a real game, and a non-champion drake kill.
  Also which name `KillerName` carries for a player: the recording sanitized them, so
  it is joined to `riotId`, `riotIdGameName` and `summonerName`. If none match, every
  drake kill reads as having no team and the pit's hints stop.
- **Table download:** the real public bucket through `r2.dev`, tested only against a fake.
- **Relaunch:** after `GameEnd` with the Tauri updater.

## Provisional values

Rechecked on the first real collection: 12,293 ranked solo matches from Silver,
Gold and Platinum, patches 16.20 and 16.21, on one region. The pilot had 529
matches, pooled by role.

Kept after the recheck:

- **Minimum n=100 and target n=300.** At n=100, 601 Cells cover 91% of player
  games, and the median at minute 20 varies by about 0.2 of the Neutral zone between
  resamples. At n=300 that falls to 0.07, with 70% coverage.
- **CS and gold Neutral zones.** From minute 15 on, 38–42% of players fall inside
  them. At minute 5 the ±8 floor puts 67% inside for CS.
- **The 15 minute filter.** Games of 15 to 20 minutes are 6% of those kept, and
  dropping them would move the median at minutes 5 and 10 by under 0.01 of the zone.
- **The drift report's bar.** At the full Neutral zone, 0 of 116 Cells with n≥100 on
  both patches shift. At half the zone, 11 do.
- **Stored size:** about 8 KB per match against the 22 KB estimate.

Still open:

- **The level Neutral zone.** ±1 holds 76% of players at minute 20 and 97% at
  minute 5, so the level Marker is almost always neutral.
- **The 20% publish gate.** Dropping 16.20 from the window now would lose 18% of
  Cells, close to the gate. A build right after a patch ships may be refused while
  the newer patch is thin.

## Out of scope

Wave and cannon Timing hints, despawn hints, monetization, Windows, post-match
statistics, data about enemy players, user accounts, and options beyond the
settings above. After the MVP, every option should become a setting, which will
need a window that takes focus.
