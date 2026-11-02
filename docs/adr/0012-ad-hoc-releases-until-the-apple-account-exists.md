---
status: accepted
---

# Ad-hoc releases until the Apple account exists

The release workflow publishes ad-hoc signed builds while the owner has no Apple Developer Program membership. This overrides the line in [ADR 0007](0007-two-release-phases.md) that unsigned builds are never handed out. The builds are for the owner's own Macs for now, and the owner accepts that anyone who finds `lollipop-releases` gets them too.

The rest of the public phase holds: the tag build, the draft and the signed updater manifest. The updater key does not depend on Apple, so updates still install silently. The workflow picks the signature from the secrets, so adding the Apple secrets makes the next tag a notarized build, with no change to the code.

## Consequences

- Gatekeeper blocks a downloaded ad-hoc app: the release notes give the `xattr` command that clears its quarantine. An update the app downloads itself carries no quarantine.
- The habit ADR 0007 wanted to avoid is back for anyone who installs one of these builds.
- The first notarized build replaces an ad-hoc one through the updater, and nothing else changes for an install.
