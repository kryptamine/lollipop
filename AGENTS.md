# AGENTS.md

Lollipop is a Tauri overlay for League of Legends. It polls Riot's Live Client
Data API and shows the active player's CS pace against a target.

## Language rule (mandatory)

**Everything written into this repository is in English**: code comments, commit
messages, log and error messages, documentation, test names, ticket text. Only
the conversation with the user follows the user's language.

## Commit convention (mandatory)

Commits follow [Conventional Commits](https://www.conventionalcommits.org):

```
<type>(<optional scope>): <subject>

<optional body explaining why, wrapped at 72 columns>
```

Types in use: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`. Scope
is an area (`live`, `ui`, `scripts`), or `deps` for a dependency update, which
is always `chore(deps)`.

- Subject in the imperative mood, lowercase, no trailing period, ≤72 characters
  ("add pace target setting", not "Added pace target setting.").
- The body explains why; the diff already says what.
- One logical change per commit.
- Commits and pull request bodies carry no co-author trailer or attribution line.

## Branches and pull requests (mandatory)

Work on a branch cut from `main`, named `<type>/<kebab-summary>` with the commit
types: `feat/pace-target-setting`, `docs/security-policy`. **`main` takes no
direct commits.**

Keep a pull request to one thing. A refactor, a feature and a formatting sweep
are three pull requests.

## Type rule (mandatory)

**A struct field exists only if code reads it.** The Live Client Data payload
carries far more than Lollipop needs, and every unread field claims a dependency
nothing exercises. Add a field in the change that reads it, and delete it with
its last reader. The same holds for methods, constants and event payloads.

## Comment rule (mandatory)

**A comment is at most three lines**, in every language in the repository, and
it says what is surprising rather than what is visible. A decision that needs a
paragraph goes in `docs/adr/`, with one line in the code pointing at it.

Before writing or editing a comment, read
[.claude/rules/comments.md](.claude/rules/comments.md).

## Commands

The gate before any commit, run from `src-tauri/`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Working here

- Non-trivial work starts from an issue, with scope settled there. Small fixes
  go straight to a pull request: a typo, an obvious bug, a dependency bump.
- Tests land with the behaviour they cover.
- The code is the source of truth, then the docs, then a comment. A doc that
  contradicts the code is a bug in the doc: fix it in the change that found it.

## Agent skills

### Issue tracker

Issues live in GitHub Issues (kryptamine/lollipop), via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
