---
status: accepted
---

# CI is off, and the gate runs locally

The GitHub Actions job no longer runs on pull requests or pushes, and can only be started by hand. `make check` is the gate, run before every push. This changes one line of [ADR 0001](0001-tauri-preact-rust-core-stack.md): linters stay strict, but nothing on GitHub blocks a pull request.

The repository is private on the free plan, so GitHub refuses branch protection and rulesets, and a red check could not stop a merge anyway. With one developer, the job only added noise and waiting at this stage.

## Consequences

- Nothing enforces the gate. A push that skips `make check`, including one with `--no-verify`, reaches `main` unchecked.
- The pre-push hook runs only `pnpm lint`, so Rust linters and tests depend on running `make check`.
- The criterion in issue #30 that linters block a pull request stays unmet.
- The workflow stays in the repository. Turning it back on means restoring its `pull_request` and `push` triggers, which should happen when a second person starts contributing or before the public phase in [ADR 0007](0007-two-release-phases.md). Required checks also need GitHub Pro or a public repository.
- The release workflow is separate: a `v*` tag starts it, and it runs no checks, so a tag goes only on a commit that passed `make check`.
