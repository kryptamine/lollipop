---
status: accepted
---

# The Build starts from items bought together

ADR 0018 took the five completed Item candidates with the largest shares, each counted on its own. Two items bought instead of each other, such as two first items, then both reach the Build, and no game builds it. So the collector now counts a usual build per champion, role and bucket: it picks the completed candidate most games bought, keeps only the games that bought it, and repeats up to five items. Once keeping only those games would leave fewer than the Item candidates' threshold, the remaining picks count over the games last kept, since a handful of games would be noise. The app starts the Build from those five, by median minute, and a table built before them falls back to shares.

## Considered options

- **Whole builds counted as sets:** five-item sets rarely repeat exactly, and games end before the fifth.
- **Ordered paths, item by item:** the order splits the games further, and the Build already orders by median minute.

## Consequences

- The Build may hold an item with a smaller share than one it leaves out.
- In thin data the picks past the threshold count over games that may hold an earlier pick's alternative, so two alternatives can still meet late in the Build.
- A usual build holds fewer than five items when no other candidate is bought with its picks, and the Build is then shorter rather than filled by share.
- `schema_version` stays 2: the app reads `usual_builds` where it is and shares where it is not.
