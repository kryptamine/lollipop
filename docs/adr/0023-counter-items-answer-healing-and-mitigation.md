---
status: accepted
---

# Counter items answer healing and mitigation

A team that heals or mitigates a lot calls for Grievous Wounds or penetration, which a Build drawn from the Damage profile alone never adds. The collector measures per champion the healing done and the damage mitigated a minute, `totalHeal` and `damageSelfMitigated` of match-v5, over the Patch window. A team summing at least the top quarter of stored teams is heavy in it. Against such a team the Build takes a Counter item from the champion's Item candidates, the one built from a component the player holds, else the one with the largest share, unless the Build or the inventory holds one: the last completed item left makes room for it, never the Next item. Once in, no other item takes its place: an off-Build item drops another one, a Build change leaves it alone, and only an off-Build item that counters the same Threat replaces it. Penetration follows the player's magic share, and a champion with none gets no penetration, as with the Elixir.

## Considered options

- **Champions named by hand:** stale with each patch and rework, and untestable against data.
- **Enemies' items, as in ADR 0019:** healing comes from kits as much as from items, and armor items arrive too late to change a Build.
- **A bucket per Threat:** three Damage profile buckets times two Threats would thin every list below the threshold.

## Consequences

- A champion whose games never buy a Counter item gets none: the Build only uses Item candidates.
- Threats come from champions alone, so a Counter item is known from the loading screen and changes only when the Build is made again.
- The top quarter is a choice, not a measurement of when a Counter item pays off.
- `damageSelfMitigated` sums armor, magic resist and other damage reduction, so a team that mitigates with magic resist still calls for armor penetration from a physical player. Match-v5 does not split it.
- `totalHeal` counts regeneration, potions and heals on allies as well as lifesteal, which lifts tanks and enchanters alike.
- With a full inventory, or no item left but the Next item, no Counter item enters: a Swap is the model's call.
