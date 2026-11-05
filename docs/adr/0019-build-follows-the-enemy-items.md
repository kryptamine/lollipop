---
status: accepted
---

# The Build follows the items enemies build

A Damage profile from champions alone misreads a team whose champion builds off its usual kind of damage: a Shaco with The Collector and Infinity Edge counted as half magic. So each enemy's magic share now averages the champion's usual share, weighed as one item, with the completed items they hold that deal damage: an item tagged `SpellDamage` counts as magic, one tagged `Damage` as physical, one with both as half each, and a tank or utility item not at all. When the team's sum crosses into another bucket, the Build is made again from that bucket's Item candidates, as at the game's start. ADR 0018 holds that a Build moving under the player is worse than one slightly off, so the move keeps what the player has put gold into: held items and a started Next item stay, the latter still first.

## Considered options

- **Items alone, once an enemy holds one:** one item flips a champion whole, and the bucket flips with it.
- **The damage enemies have dealt:** Live Client Data does not give it.
- **Change only the candidates a model may pick from:** without an OpenAI key the player would never see it.

## Consequences

- An enemy finishing an item can remake the Build mid-game, each time the sum crosses a bucket boundary.
- An item dropped earlier, by a Build change or an off-build item, may come back when the Build is made again.
