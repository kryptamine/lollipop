---
status: accepted
---

# Public and local Riot APIs only

Lollipop reads Live Client Data, the League client API and the Riot web API, and nothing else. It does not read process memory, intercept traffic, inject into the game or evade anti-cheat. A public release is the goal, so Riot's policies apply from day one.

Markers are the main grey area in those policies ("may highlight" against "drawing conclusions for you during gameplay"), so they stay measurements with no instructions. These are readings of the policies, not Riot rulings, and only the product audit is binding.

## Considered options

- **OCR for an exact CS:** Riot's pages never mention it and its data-source wording points away from it, so it needs asking Riot first.
- **Rebuilding exact CS from gold increases:** works around a value Riot coarsens on purpose.
- **Overwolf:** no macOS overlays, and its overlay is injected.

## Consequences

- CS stays floored to 10, so its value cannot match Dota Plus. The Marker colour holds by comparing at the bucket midpoint.
- The League client API is not officially supported, so tier, queue and patch can break without notice.
- Wave and cannon Timing hints have no policy precedent, which is one reason they are out of the MVP.
