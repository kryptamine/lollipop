---
status: accepted
---

# A standing Build replaces item Advice

The overlay always shows a Build: code orders the Item candidates of the enemy team's Damage profile, boots with the largest share and the five completed items with the largest shares, by median minute. A model may replace only the Next item, on the loading screen or a death, before the player starts it, and says why in a Build change. Advice no longer names items. Item Advice from ADR 0017 named one item at a Trigger, so the player never saw where the build was going and an item vanished from view between Triggers.

## Considered options

- **Builds players actually complete, counted by the collector:** truer than shares taken one item at a time, which can put two alternative first items in one Build, but needs a schema and collector change. Left for later.
- **The model writes the whole Build:** less predictable, and a Build that moves under the player is worse than one slightly off.
- **The model may change any unstarted item:** one shared basic component starts several items at once, so only the Next item is open.

## Consequences

- ADR 0004 still holds: a replacement outside the Item candidates is dropped with its Build change.
- Without an OpenAI key the Build still shows; Build changes, Swaps and the model's Elixir pick need one.
- The Back to base Trigger goes, since all it advised was an item.
