---
status: accepted
---

# No Marker beats a wrong Marker

When the app does not know something a comparison needs, it shows nothing and says why. Role and tier are detected automatically, with no manual choice and no guessing. The overlay's value depends on trusting its colours, and a player cannot tell when a Marker is measured against the wrong group.

## Considered options

- **Manual role and tier:** a settings surface for a value the API usually provides, and a way to be compared to the wrong Cell without noticing.
- **Guessing the role from the champion's most common role:** wrong exactly when the pick is unusual.
- **An all-tier Benchmark for an unknown tier:** nothing would tell the player that the Marker is against another group.

## Consequences

- An unranked player gets no Markers, and an LCU break removes them for everyone until it is fixed. Revisit this before public release.
- A lane swap is a known MVP limit if the API keeps reporting the assigned role.
- The same rule covers Timing hints: an unknown queue or patch, or a new Season without a schedule, shows no hints.
- The same rule covers Advice: a malformed or over-length answer, or an item outside the candidates, drops the whole Advice, and an OpenAI that cannot be reached says so once per game.
