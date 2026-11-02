---
status: accepted
---

# The collector archives Match-V5 payloads

Each kept match also stores its match and timeline bodies, zstd-compressed, in `match_payloads`. A statistic no table reads yet, such as K/D/A or item purchases, can then be built from matches already collected, instead of waiting a Patch window of new ones after a schema change. The archive is bytes, so no struct gains a field nothing reads.

Two parts of a body are removed before it is stored. Every key that names a player goes, as PUUIDs are never stored. The per-minute `championStats` and `damageStats` of each timeline frame go too: they are most of a timeline's size, and nothing planned reads them.

## Considered options

- **Columns for the next statistics now:** a guess at what will be read, and fields no code reads.
- **Whole bodies:** an estimated 30 to 50 GB for a full window, against about 5 GB without the archive.
- **Refetching timelines later:** a call per stored match, taken from collection on a personal key.

## Consequences

- The archive goes with its match when the Patch window shifts, so it never outgrows one window. The estimate is 10 to 15 GB for a full window, untested on real bodies.
- Matches stored before the archive have no row in it. A table built from the archive waits until the window holds archived matches only.
- A statistic that needs a removed frame stat has to wait for new matches after its removal is undone.
