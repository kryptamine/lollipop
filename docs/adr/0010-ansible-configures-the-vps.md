---
status: accepted
---

# Ansible configures the VPS

The collector's VPS is configured by a playbook in `deploy/`, so that moving to another host is one command rather than a list of manual steps. Ansible needs nothing on the host beyond SSH and Python, and running it twice changes nothing. Terraform would only create the VPS, which is one manual step on one host, and a shell script would have to build idempotence by hand.

Secrets live in `deploy/inventory.yml`, which git ignores, and not in the repository even encrypted. The repository is private today, but [ADR 0007](0007-two-release-phases.md) plans a public phase, and encrypted keys in its history would outlive any rotation.

The binary is cross-compiled in Docker to a static musl target for the host's architecture, because a small VPS may not have the memory to build it, and a static binary does not depend on the host's glibc.

## Consequences

- Losing `deploy/inventory.yml` means collecting the secrets again from Riot, GitHub and Cloudflare. No copy of it exists elsewhere.
- The playbook does not create the VPS, and does not check that the collector stays up after starting.
- `lollipop-restore.service` runs ahead of the collector whenever the database is missing, so the same path serves a host move and a recovery, and each move exercises the backup.
