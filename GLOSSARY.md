# Lollipop

An overlay for League of Legends that shows the active player how their game
compares, minute by minute, to how similar games usually go.

## Language

**Advice**:
A recommendation a model writes for the active player, such as a game plan. It never names an item: the Build does.
_Avoid_: Hint, tip, suggestion

**Benchmark**:
The expected value of a metric at a given game minute, derived from match statistics.
_Avoid_: Target, goal, norm

**Benchmark table**:
All the Benchmarks from one collector run, published together.
_Avoid_: Dataset, baseline

**Build**:
The completed items and boots the player is advised to buy, in the order to buy them, less those already held. It drops an item once the player finishes another one in its place, and only its Next item can be changed, until the player starts it.
_Avoid_: Item options, recommended items, build path

**Build change**:
A note of why a model replaced the Next item, shown beside the Build.
_Avoid_: Build update, item alert

**Cell**:
One champion, role and tier combination, the group of games a player is compared to.
_Avoid_: Bucket, segment, cohort

**CS pace**:
CS a minute over at least the last three minutes, against what the Cell gains over the same stretch. It shows a player who stopped farming, which total CS hides once the lead is banked.
_Avoid_: CS rate, farm speed, CS per minute

**Counter item**:
An Item candidate that answers a Threat the enemy team is heavy in: Grievous Wounds for healing, penetration of the player's damage for mitigation.
_Avoid_: Counter build, situational item

**Damage profile**:
How much of a team's damage to champions is magic, from what its champions usually deal and the damage items they hold.
_Avoid_: Damage type, AP/AD split

**Elixir**:
A 500-gold consumable of Iron, Sorcery or Wrath, worth buying once the inventory is full and before Baron or Elder.
_Avoid_: Potion

**Gold earned**:
All gold a player has received in a game, starting gold included. Spending never lowers it, and selling an item never raises it.
_Avoid_: Net worth, total gold

**Item candidate**:
A completed item or boots that players of a champion and role commonly buy against a Damage profile. A Build holds no other item.
_Avoid_: Recommended item, suggestion

**Lane opponent**:
The enemy, or for the bottom lane the enemy pair, playing the same position as the active player.
_Avoid_: Matchup, counterpart

**Lesson**:
One thing a model says the player should do differently next game, grounded in a Recap. A Recap has at most three, most costly first.
_Avoid_: Takeaway, tip, insight

**Marker**:
The glyph beside a metric: a green or red triangle when the player is above or below its Benchmark by more than the Neutral zone, and nothing within it.
_Avoid_: Arrow, indicator, delta

**Neutral zone**:
The distance from a Benchmark within which a metric shows no triangle.
_Avoid_: Threshold, tolerance

**Next item**:
The first item of the Build. It counts as started once the player buys one of its components while it leads.
_Avoid_: Item to build, target item

**Objective schedule**:
The times at which objectives spawn and respawn, which Timing hints count down to.
_Avoid_: Timers, spawn table

**Patch window**:
The patches whose matches a Benchmark table is built from.

**Proposal**:
What a Trigger's answer may change in the Build: the Next item, a Swap or an Elixir. Which ones it may propose depends on the Trigger.
_Avoid_: Capability, build proposal

**Recap**:
How one game went against its Cell, minute by minute, with the player's deaths, the stretches of slow farm and its Lessons. It is made once the game ends and shown in a window of its own.
_Avoid_: Post-game review, summary, report

**Stale table**:
A Benchmark table that cannot serve the patch of the game being played.
_Avoid_: Outdated, expired

**Swap**:
What the Build becomes once the inventory is full: sell one finished item, never boots, for an Item candidate the player lacks.
_Avoid_: Rebuild, replacement

**Threat**:
What an enemy team may do a lot of that a Counter item answers: healing, or damage mitigated. A team is heavy in it from the top quarter of stored teams.
_Avoid_: Danger, enemy profile

**Threat profile**:
Per champion, the amount of one Threat a minute, and the team sum from which a team is heavy in it.
_Avoid_: Heal table, tankiness score

**Timing hint**:
A pop-up shown shortly before an objective spawns.
_Avoid_: Timer, alert, notification

**Turning point**:
A moment that changes the game plan, such as an objective taken, an inhibitor down, an Ace or the first turret of the game.
_Avoid_: Periodic, milestone

**Trigger**:
A moment in a game that asks for Advice, such as the loading screen or the player's death.
_Avoid_: Event, hook

**Usual build**:
The completed items games of a champion and role buy together against a Damage profile, which the Build starts from.
_Avoid_: Build path, core build

**Voiceover**:
Advice read aloud.
_Avoid_: TTS, narration

**Widget**:
A self-contained element of the overlay, such as the strip of Markers or the Timing hint banner.
_Avoid_: Panel, module
