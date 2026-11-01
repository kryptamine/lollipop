# Lollipop

An overlay for League of Legends that shows the active player how their game
compares, minute by minute, to how similar games usually go.

## Language

**Benchmark**:
The expected value of a metric at a given game minute, derived from match statistics.
_Avoid_: Target, goal, norm

**Benchmark table**:
All the Benchmarks from one build, published together.
_Avoid_: Dataset, baseline

**Cell**:
One champion, role and tier combination, the group of games a player is compared to.
_Avoid_: Bucket, segment, cohort

**Marker**:
The green or red triangle beside a metric, showing whether the player is above or below its Benchmark.
_Avoid_: Arrow, indicator, delta

**Patch window**:
The patches whose matches a Benchmark table is built from.

**Stale table**:
A Benchmark table that cannot serve the patch of the game being played.
_Avoid_: Outdated, expired

**Timing hint**:
A pop-up shown shortly before an objective spawns.
_Avoid_: Timer, alert, notification

**Widget**:
A self-contained element of the overlay, such as the strip of Markers or the Timing hint banner.
_Avoid_: Panel, module
