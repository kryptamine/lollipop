---
status: accepted
---

# The Enemy tracker infers from what every player sees

The Enemy tracker reads only what Live Client Data already gives every player through the scoreboard and the kill feed: who is dead and for how long, what each enemy holds, and who took part in each kill and objective. From these it infers a Headcount, a Jungler sighting and a Recall. It never reads positions, which Live Client Data does not have, so every line says when it was true rather than where an enemy is now. ADR 0008's data sources still hold, and like ADR 0013 the owner takes the Riot policy risk of drawing these conclusions during play.

## Considered options

- **Minimap reading for real positions:** it is screen capture plus OCR, which ADR 0008 rules out, and is the line between a scout and a map hack.
- **Jungle camp timers from CS:** CS is floored to 10, so a camp's 4 CS cannot be told from a wave's, and a wrong camp is worse than none (ADR 0004).
- **Turret kills as sightings:** the lane in a turret's name is not confirmed by any recording.

## Consequences

- A Recall is seen only once the enemy buys; a trip to base with no purchase goes unseen.
- Items that turn into another in lane, such as Muramana or a finished support quest, are listed by id so they do not read as a Recall. A new field upgrade needs adding there.
- A Jungler sighting older than 90 s is dropped, as the jungler can be anywhere by then.
- Voiceover does not read the Enemy tracker: its lines are English while the voice follows the Advice language.
