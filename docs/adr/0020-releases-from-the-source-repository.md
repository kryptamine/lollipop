---
status: accepted
---

# Releases from the source repository

The source repository is public now, so releases live on its own GitHub Releases and `lollipop-releases` is retired. This overrides the separate repository in [ADR 0007](0007-two-release-phases.md), which existed only to keep the source private. The release workflow publishes with the repository's own token, so the personal token for the other repository goes.

Release-please opens the release as a draft and creates its tag at once, and the build attaches the DMG, the archive and `latest.json` to that draft. A published release-please release would become the latest before its files exist, and the updater would read a missing manifest.

## Consequences

- Installs of 0.1.1 and older poll `lollipop-releases`. The first release from here also goes there by hand, as a `latest.json` pointing at this repository, and then that repository is archived.
- A release that is not an app build would become the latest and hide the manifest, so the repository carries app releases only.
