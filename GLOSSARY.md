# Lollipop

An overlay for League of Legends that shows the active player how their game
compares, minute by minute, to how similar games usually go.

## Language

**Advice**:
A recommendation a model writes for the active player, such as a game plan or an item to buy.
_Avoid_: Hint, tip, suggestion

**Back to base**:
A visit the player makes to the shop, alive or dead, seen by its first purchase.
_Avoid_: Recall

**Benchmark**:
The expected value of a metric at a given game minute, derived from match statistics.
_Avoid_: Target, goal, norm

**Benchmark table**:
All the Benchmarks from one build, published together.
_Avoid_: Dataset, baseline

**Cell**:
One champion, role and tier combination, the group of games a player is compared to.
_Avoid_: Bucket, segment, cohort

**Damage profile**:
How much of a team's damage to champions is magic, from what its champions usually deal.
_Avoid_: Damage type, AP/AD split

**Gold earned**:
All gold a player has received in a game, starting gold included. Spending never lowers it, and selling an item never raises it.
_Avoid_: Net worth, total gold

**Item candidate**:
A completed item or boots that players of a champion and role commonly buy against a Damage profile. Advice recommends no other item.
_Avoid_: Recommended item, suggestion

**Item options**:
The three Item candidates code ranks first for a Trigger's Advice: the one the player's components build into most, then the latest item in progress (shown by earlier Advice, not bought, and not left for another finished item), then the earliest by median minute among those not bought before a held candidate, else the latest of those. None once the inventory is full.
_Avoid_: Shortlist, top picks

**Item to build**:
The Item option a Trigger's Advice names, picked by the model: the first unless the enemy team's items, levels or the kill score call for another.
_Avoid_: Next item, target item

**Lane opponent**:
The enemy, or for the bottom lane the enemy pair, playing the same position as the active player.
_Avoid_: Matchup, counterpart

**Marker**:
The glyph beside a metric: a green or red triangle when the player is above or below its Benchmark by more than the Neutral zone, and nothing within it.
_Avoid_: Arrow, indicator, delta

**Neutral zone**:
The distance from a Benchmark within which a metric shows no triangle.
_Avoid_: Threshold, tolerance

**Objective schedule**:
The times at which objectives spawn and respawn, which Timing hints count down to.
_Avoid_: Timers, spawn table

**Patch window**:
The patches whose matches a Benchmark table is built from.

**Stale table**:
A Benchmark table that cannot serve the patch of the game being played.
_Avoid_: Outdated, expired

**Timing hint**:
A pop-up shown shortly before an objective spawns.
_Avoid_: Timer, alert, notification

**Turning point**:
A moment that changes the game plan, such as an objective taken, an inhibitor down, an Ace or the first turret of the game.
_Avoid_: Periodic, milestone

**Trigger**:
A moment in a game that asks for Advice, such as the loading screen or the player's death.
_Avoid_: Event, hook

**Voiceover**:
Advice read aloud.
_Avoid_: TTS, narration

**Widget**:
A self-contained element of the overlay, such as the strip of Markers or the Timing hint banner.
_Avoid_: Panel, module
