---
status: accepted, its releases repository superseded by ADR 0020
---

# Two release phases, split at the first other person

The owner builds a local ad-hoc DMG. Anyone else gets a notarized Developer ID DMG from a separate public repository, `lollipop-releases`, with silent updates. There is no "friends" phase: unsigned builds are never handed out, because "Open Anyway" trains players to bypass Gatekeeper and Apple does not guarantee it stays available.

The separate repository keeps the source private, so opening it stays its own decision. Benchmark tables stay on their own channel, so the VPS has no access to app releases.

## Consequences

- Updates install without asking, since the overlay takes no clicks. A bad release is fixed by a newer version, with no rollback.
- The manifest is signed from the first public build, because adding it later would need every install updated first.
- A compromised GitHub account yields both the Apple identity and the updater key. This risk is accepted.
