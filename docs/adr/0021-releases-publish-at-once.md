---
status: accepted
---

# Releases publish at once

Merging the release pull request ships the version: the build attaches its files to the draft and then publishes it. This overrides the hand-published draft of [ADR 0007](0007-two-release-phases.md) and moves the real-match checklist of [ADR 0001](0001-tauri-preact-rust-core-stack.md) after the release. The installs are the owner's own Macs, and a hand step per release cost more than a bad build does.

The release stays a draft until its last file is attached, so the updater never reads a latest release without `latest.json`.

## Consequences

- A bad build reaches every install within minutes, with no rollback: a newer version fixes it.
- The checklist finds problems after the players have the build. Once other people have installs, this decision is worth revisiting.
