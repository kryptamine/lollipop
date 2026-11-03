---
status: accepted
---

# The model picks the item among code's Item options

Code ranks the Item candidates and gives the model the first three as Item options; the model names one, the first unless the enemy team's items, levels or the kill score call for another. Before this, code held the one Item to build and the model only explained it, so nothing the game showed beyond the inventory and the enemy champions could change the item.

## Considered options

- **Code alone, as before:** predictable, but blind to the enemy team's items and the score, which the model already sees in the prompt.
- **Every candidate left:** a longer prompt and more room for an odd pick late in a build.
- **A Damage profile from the enemy team's live items:** needs item stats in the table and a collector change; left for later.

## Consequences

- ADR 0004 still holds: an `item` outside the options drops the whole Advice.
- Item Advice can differ between two games with the same inventory, and a test can only check that the pick is one of the options.
