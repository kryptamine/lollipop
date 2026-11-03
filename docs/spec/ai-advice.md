# AI Advice

Advice is one short idea a model writes for the player at a Trigger, shown in the
Advice Widget and, optionally, read aloud by Voiceover. It is built on the
player's own OpenAI key. Terms follow [`GLOSSARY.md`](../../GLOSSARY.md); the
MVP it builds on is [`mvp-core.md`](mvp-core.md). Each section links the issue
that settled it, where the reasoning lives.

Advice gives instructions, which ADR 0008 ruled out; the owner accepts that risk
([ADR 0013](../adr/0013-advice-accepts-riot-policy-risk.md)). Markers stay
measurements. No Advice beats a wrong one: anything Lollipop cannot check is
dropped, never shown ([ADR 0004](../adr/0004-no-marker-beats-a-wrong-marker.md)).

## Settings window

Settled in [#91](https://github.com/kryptamine/lollipop/issues/91) and
[#103](https://github.com/kryptamine/lollipop/issues/103);
[ADR 0014](../adr/0014-settings-window-takes-focus-on-demand.md) supersedes ADR 0006
in part. This replaces the switches in the MVP's menu bar.

- **Menu bar:** the status line, "Settings…" (Cmd-,) and Quit. Nothing else.
- **Opening:** only from "Settings…", never by itself, first launch included. A
  second click raises the open window. The app stays `Accessory`. Opening it in a
  game hides the overlay, accepted.
- **Closing:** hides the app so focus returns to where it came from; the overlay
  comes back when that is League.
- **Drawing:** a second Tauri window drawn by the Preact view. A macOS toolbar
  with an icon over each label; grouped rows on rounded panels, label left and
  control right, a small caption above each group.
- **Changes apply at once**, in a running game too. No Save button. Settings stay
  in `settings.json`; the OpenAI key never does.

| Tab | Rows |
| --- | --- |
| General | Launch at login |
| Overlay | Marker strip, Timing hints, Advice Widget |
| Advice | OpenAI: API key · Advice: Language, Loading screen, Death, Back to base, Turning points · Voiceover: Read Advice aloud |

- **Advice Widget switch** is the only Advice on/off. Off: no OpenAI calls. It
  defaults on like every Widget, but is disabled with no key, subtitled "Add an
  OpenAI key in the Advice tab".
- **Advice Widget off:** a yellow banner between the OpenAI and Advice groups,
  "The Advice Widget is off. Turn it on in Overlay", the link switching tabs. The
  Advice and Voiceover groups grey out and stay visible; the key stays editable.
- **Language:** system default, ru or en. Every Trigger uses it.
- **Trigger switches:** four, all on by default.
- **Disabled reasons** show as the row's subtitle.

## OpenAI key

Settled in [#92](https://github.com/kryptamine/lollipop/issues/92);
[ADR 0016](../adr/0016-player-openai-key-in-the-keychain.md).

- **Entry:** a secure field on the right of the "API key" row. A paste or Enter
  starts the check at once, with a spinner beside it.
- **Check:** `GET /v1/models/gpt-6-luna`. Only a `200` stores the key in the
  macOS Keychain. A `401` shows "OpenAI rejected the key"; a network error or
  `5xx` shows "Could not reach OpenAI, try again". Neither stores anything.
- **Display:** `sk-…abcd`, the last four kept at save, with a "Remove key" link.
  The full key never returns to the view. Pasting a new key replaces it.
- **Removal:** no confirmation. Deletes the Keychain item and turns the Advice
  Widget off.
- **Cost line:** under the key row, "About $0.005 per game, billed to your OpenAI
  account", linking to OpenAI's usage page. Updated with the code when the model
  changes ([#124](https://github.com/kryptamine/lollipop/issues/124)).
- **Ownership:** `app` reads the Keychain (`security-framework`, no `unsafe`) at
  start and after a change and hands the key to `core` as a value. Never in
  `settings.json` or logs.
- **Keychain prompt:** an ad-hoc build's signature changes every release, so macOS
  asks once after each update. Accepted until notarized builds.
- **Dev:** a debug build takes the key and base URL from env vars for
  `make dev-mock`; release ignores them.

## OpenAI call

Settled in [#88](https://github.com/kryptamine/lollipop/issues/88), measured in
[#99](https://github.com/kryptamine/lollipop/issues/99). Research:
[`openai-api-for-advice.md`](../research/openai-api-for-advice.md).

- **Model:** `gpt-6-luna`, fixed in code. OpenAI only.
- **Request:** `POST /v1/responses` from `core`, plain `reqwest` with private serde
  structs, `store: false`, `reasoning.effort: "none"`, `max_output_tokens: 120`,
  and a strict `json_schema` that is the same on every call: a nullable `advice`
  string and a nullable `item` id. The base URL is configurable for the fake.
- **Prompt:** never Riot IDs or summoner names, never enemy ranks.
- **Measured:** about 1,950 input tokens, median 1.4 s, worst 3.7 s (first calls).
  Prompt caching applies by itself. About $0.005 per game.
- **Answer checks:** JSON outside the schema, `advice` over 120 characters, or an
  `item` outside the Trigger's candidates drops the **whole** Advice, with no retry
  and no truncation. A null `advice` shows nothing.
- **Per game:** at most 100 OpenAI requests, retries included. Past that, Advice is
  dropped silently for the rest of the game, with one log line. Not a setting.
- **No spend cap or counter:** spend limits live in the player's OpenAI project.
  Lollipop reads nothing about cost from responses.

### Failures

Settled in [#123](https://github.com/kryptamine/lollipop/issues/123).

- **Timeout:** min(8 s, time left before the Trigger's drop deadline).
- **Retry:** one, about 1 s later, on a network error, timeout, `5xx` or `429`
  rate limit, only with at least 4 s left before the deadline. A `Retry-After`
  longer than that gets none. `429 insufficient_quota` is not retried.
- **Unreachable:** after the retry, the Widget says once per game "Advice
  unavailable: could not reach OpenAI", ranked as Advice of the Trigger that failed.
- **Key fails in a game:** a `401` or `429 insufficient_quota` stops calls until a
  new key is saved. The Widget says why once; the Advice tab marks the key with the
  same status. The key is not deleted.
- **No circuit breaker:** every Trigger tries on its own after a failure.
- **Log:** Trigger, failure kind (network, timeout, HTTP status, schema, length,
  candidate), latency, whether retried. Never the key, the prompt or the answer.

## Advice Widget

Settled in [#93](https://github.com/kryptamine/lollipop/issues/93), amended by
[#126](https://github.com/kryptamine/lollipop/issues/126).

- At the right edge above the minimap, a card about 300 px wide.
- One Advice at a time, at most 120 characters, shown for 20 s.
- With an item, a ~32 px icon left of the text, from
  `ddragon.leagueoflegends.com/cdn/{ver}/img/item/{id}.png`, the version taken from
  the Benchmark table. The CSP gains that `img-src`. A failed load shows the Advice
  without an icon. The item's name is in the text.
- Shown beside the Timing hint banner; both stay visible.
- Like every Widget, it shows only while the overlay does.

### Sharing the Widget

Settled in [#121](https://github.com/kryptamine/lollipop/issues/121).

- **Rank:** Death > Back to base > Turning point > Loading screen. An answer of the
  same or higher rank replaces the Advice showing; a lower one is dropped.
- Rank is checked when the answer arrives. A lower request in flight is not
  cancelled.
- No cooldown across Triggers: each Trigger's own rules plus rank.
- Voiceover follows the Widget, so it inherits the rank.

## Triggers

What each Trigger can read is in [#87](https://github.com/kryptamine/lollipop/issues/87)
and [`trigger-game-data.md`](../research/trigger-game-data.md). Every prompt also
carries the Advice language. "Dropped" means the answer is not shown; once shown,
Advice stays the Widget's 20 s.

| Trigger | Item | Drop deadline |
| --- | --- | --- |
| Loading screen | Never | 1:30 `gameTime` |
| Death | Optional | Respawn |
| Back to base | Optional | 10 s after the purchase, or a death |
| Turning point | Never | 15 s after the request, or a death |

### Loading screen

Settled in [#94](https://github.com/kryptamine/lollipop/issues/94).

- **Says:** a lane plan against the Lane opponent, one idea.
- **Lane opponent:** the enemy in the same position; for `BOTTOM` and `UTILITY` the
  enemy `BOTTOM`+`UTILITY` pair, with the player's own lane partner in the prompt;
  for `JUNGLE` the enemy jungler.
- **Fires:** once per game, on the first `allgamedata` `200` that has the active
  player and no `GameStart`. Starting the app mid-game skips it.
- **None** without a Lane opponent: position `NONE`, or no enemy in that position.
- **Prompt:** all ten champions with positions, keystones and rune trees, summoner
  spells; the player's full runes; the player's tier from the LCU.
- **Shows** until `GameStart` plus 20 s; the overlay is hidden on the loading
  screen, so it is seen in the first 20 s of the game.

### Death

Settled in [#95](https://github.com/kryptamine/lollipop/issues/95).

- **Says:** why the player died and what to change, naming the
  [Item to build](#item-to-build). The model never picks another item.
- **Fires:** on a `ChampionKill` whose `VictimName` is the active player and whose
  killer is an enemy champion. A death to a turret, minion or monster gets none.
- **Prompt:** the player's champion, role, level, items and `currentGold`; the
  killer and assisters (champion, position, level, items, KDA); the whole enemy
  team's champions, items and levels; `gameTime`; how many times this killer has
  killed the player; the Item to build; items earlier Advice in this game
  recommended, and whether the player bought them.
- **Item:** the [Item to build](#item-to-build), held by code. Not filtered by gold:
  the model sees `currentGold` and may say to build toward it.
- **Another death:** sends a new request, whose answer replaces the one showing.

### Back to base

Settled in [#96](https://github.com/kryptamine/lollipop/issues/96).

- **Says:** the Item to build. One the player can afford now is "buy it now",
  otherwise "build toward it". `advice` is null when nothing changed since the
  last Advice.
- **Fires:** on the first purchase of a visit: a new item in `items` and
  `currentGold` down by at least 300 in the same poll, while alive. The 300 floor
  drops wards, potions and trinket swaps; an undo raises gold and never fires.
- **Does not fire:** before 1:30 `gameTime`; while `isDead`; within 15 s after a
  respawn; within 30 s after the last Back to base. A recall that buys nothing is
  not visible.
- **Prompt:** the player's champion, role, level, items, KDA and `currentGold`; the
  items bought in this visit; the whole enemy team's champions, items, levels and
  KDA; `gameTime`; the Item to build; earlier recommended items and whether bought.
- **Item:** as for death. The first one comes on the first Back to base.

### Turning point

Settled in [#97](https://github.com/kryptamine/lollipop/issues/97), amended by
[#121](https://github.com/kryptamine/lollipop/issues/121) and
[#153](https://github.com/kryptamine/lollipop/issues/153).

- **Says:** one macro idea for the next few minutes: which objective to play for,
  what to prioritise. `advice` is null when nothing is worth saying. Live Client
  Data has no map positions, so the instructions say the player's position is
  unknown: no lane and no named ally to join.
- **Fires on,** either team: end of laning (the first turret kill or 14:00
  `gameTime`, whichever comes first, once per game); `DragonKill` (Elder included),
  `HeraldKill`, `BaronKill`; `InhibKilled`; `Ace`; the player's own level 6, 11
  and 16.
- **Does not fire on:** later turrets, Void Grubs, other players' kills,
  multikills, first blood, inhibitor respawns.
- **Spacing:** at least 3 min from the last Turning point **shown**. A moment inside
  the window is skipped, not queued. One dropped by rank, by lateness or as null
  does not start the window. Within 30 s of a Turning point, a new one sends a new
  request whose answer replaces it.
- **In a fight:** a moment within 20 s of a `ChampionKill` the player, alive,
  killed or assisted in is held, not dropped, and asked once those 20 s pass. A
  newer moment replaces the held one. A death does not count: a dead player is out
  of the fight.
- **While dead:** fires as usual; rank keeps it off a death Advice.
- **Prompt:** the player's champion, role, level, items, KDA and `currentGold`;
  both teams' champions, items, levels and KDA; which moment fired and which team
  took it; the objective score from `eventdata` (dragons by type per team, Herald,
  Baron, turrets and inhibitors per side); `gameTime`; the texts of earlier Advice
  in this game. No candidates.
- **Dropped** also after the player dies, when they were alive at the moment.

## Item candidates

Settled in [#126](https://github.com/kryptamine/lollipop/issues/126). Research:
[`item-statistics.md`](../research/item-statistics.md).

Item Advice is hybrid: the Benchmark table supplies the candidates, code picks the
Item to build among them, the model explains it, and code rejects anything else.

- **In the app:** sum the five enemies' magic share from `allPlayers` and read the
  Damage profile bucket. The candidates are the list for the player's champion,
  role and bucket, minus items the player owns, `owned_as` included, and minus
  every boots once the player owns any id of `boots`.

### Item to build

Settled in [#152](https://github.com/kryptamine/lollipop/issues/152).

- **In progress first:** the oldest candidate earlier Advice showed and the player
  has not bought. A death never breaks it, whatever killed the player.
- **Then components:** the candidate whose `builds_from` the player's items hit
  most, ties to the earlier `minute`.
- **Else** the candidate with the earliest `minute`, ties to the larger `share`.
- **None** when no candidate is left, and `item` stays null.

### In the Benchmark table

- **Same file,** `schema_version` 2, published under `v2/`, with one manifest and
  one signature, never swapped mid-game. v1 gets no new tables.
- **No extra Riot calls:** `build-table` reads the match and timeline payloads the
  collector archives. A purchase is an `ITEM_PURCHASED` not taken back by an
  `ITEM_UNDO`. A match stored before archiving gives no candidates. Final builds
  matter only for transforms, below.
- **Key:** champion + role, no tier, in `item_candidates` beside `cells`, one entry
  with `physical_heavy`, `mixed` and `magic_heavy` lists. `cells` stay as they are.
- **Damage profile:** `damage_profile.magic_share`, per champion magic / total damage
  dealt to champions over the Patch window. A team's profile is the sum of its five
  shares; `mixed_from` and `magic_heavy_from` are terciles of that sum over every
  stored team, each the lowest sum of its bucket. A champion with no share counts 0.
- **Fields:** `item` (id), `share` (games bought, in that bucket), `minute` (median
  first purchase). No slot, no win rate.
- **List:** completed items up to 90% coverage, at most 12, plus tier-2 boots up to
  90%, at most 3, in one list by share. Classes come from Data Dragon's `item.json`
  for the newest patch of the window: completed is on map 11, purchasable, no `into`, at
  least 2200 gold, and not boots, a consumable or a trinket; tier-2 boots have the
  `Boots` tag and `depth` 2. A build that cannot reach Data Dragon writes no table.
- **Thin groups:** a bucket under 50 games uses the champion+role list pooled over
  buckets; under 50 pooled, no entry and `item` stays null. Threshold in
  `table_config`, and `--min-candidate-games` on `build-table` and `publish`.
- **Transforms:** Seraph's, Muramana and support upgrades map to their purchasable
  source through Data Dragon `specialRecipe`, else a lone `from`; a candidate carries
  optional `owned_as` ids. A completed item whose `into` holds only transforms (items
  of map 11 no shop sells) stays completed.
- **Components:** a candidate carries optional `builds_from`, every id down its
  Data Dragon `from` tree. A table without it ties no component to a candidate.
- **Boots:** `boots` lists every Data Dragon item of map 11 with the `Boots` tag and
  `depth` 2 or more, candidate or not, since a player holds one pair. A table
  without it leaves boots among the candidates.
- **Names:** `item_names` maps every candidate id to its `en` and `ru` name, from
  Data Dragon at the newest patch of the Patch window; a name with no `ru_RU`
  translation repeats the English one. `data_dragon_version` is that version.

## Voiceover

Settled in [#98](https://github.com/kryptamine/lollipop/issues/98). Research:
[`voiceover-options.md`](../research/voiceover-options.md).

- **Engine:** macOS `AVSpeechSynthesizer` through `objc2-avf-audio`, in
  `platform`. Free, offline, no focus taken.
- **Voice:** the best installed voice for the Advice language (premium > enhanced >
  default). No picker. A footer line says better voices download in System
  Settings → Accessibility → Spoken Content.
- **Switch:** one, "Read Advice aloud", off by default. Disabled with "Needs the
  Advice Widget" when the Widget is off, or "No voice installed for this language".
- **Volume:** 1.0 on the system volume. League's sound is not ducked.
- **Interruption:** speech follows the Widget. A new Advice stops the reading at a
  word boundary and reads the new one; a drop or the Widget hiding stops it. No
  queue.
- **Warm-up:** a short line at volume 0 when a game starts hides the cold voice
  load (239-620 ms measured).

## Tests

Settled in [#125](https://github.com/kryptamine/lollipop/issues/125).

- OpenAI is an external process, faked in `core` tests through the configurable
  base URL, with a fake key. Each rule above that code decides (non-candidate,
  over-length or malformed answer, retry, timeout, 100-request cap, Trigger rank and
  deadlines) gets a synthetic scenario.
- Advice wording is checked by the owner in real games. No prompt snapshots, eval
  script or release-checklist entry.

## Implementation

One pull request each, in this order of blocking:

1. [Open a settings window from the menu bar](https://github.com/kryptamine/lollipop/issues/139)
2. [Take the player's OpenAI key in the Advice tab](https://github.com/kryptamine/lollipop/issues/130), after 1
3. [Add item candidates to Benchmark table schema 2](https://github.com/kryptamine/lollipop/issues/131)
4. [Add item names, transforms and Data Dragon version](https://github.com/kryptamine/lollipop/issues/132), after 3
5. [Give Advice on the loading screen](https://github.com/kryptamine/lollipop/issues/133), after 2
6. [Handle OpenAI failures and cap requests per game](https://github.com/kryptamine/lollipop/issues/134), after 5
7. [Give Advice on death with an item candidate](https://github.com/kryptamine/lollipop/issues/135), after 4 and 5
8. [Give Advice on a back to base](https://github.com/kryptamine/lollipop/issues/136), after 7
9. [Give Advice on a Turning point](https://github.com/kryptamine/lollipop/issues/137), after 5
10. [Read Advice aloud with Voiceover](https://github.com/kryptamine/lollipop/issues/138), after 5

## Out of scope

- Champ select Advice and post-game review.
- Choosing the model, and providers other than OpenAI.
- An OpenAI voice for Voiceover: its Speech models shut down 2027-01-06.
- Researching Riot policy ([ADR 0013](../adr/0013-advice-accepts-riot-policy-risk.md)).
