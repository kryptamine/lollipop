# AGENTS.md

Lollipop is a Tauri overlay for League of Legends. It polls Riot's Live Client
Data API and shows the active player's CS.

## Layout

- `crates/core` (`lollipop-core`): domain and API polling. No Tauri.
- `crates/app` (`lollipop`): the Tauri shell. `platform` is the only module
  allowed `unsafe`.
- `crates/collector` (`lollipop-collector`): the Benchmark CLI, a stub so far.
- `ui/`: the Preact view. It draws state it is given, and only `ui/src/bridge.ts`
  may import Tauri.

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

## File rule (mandatory)

**One Rust file holds one concept: a type with its impls, or one piece of
behaviour.** Split by meaning, and keep the public path stable with `pub use` in
the parent. A trait may share a file with its one production implementation, and
private structs mapping a single JSON document may share one file. No linter
checks this.

## Test rule (mandatory)

**Tests are integration tests through an external boundary, with no unit tests.**
Only external processes (League, Riot API) are faked. `core` tests live in
`crates/core/tests/`, against a local HTTP server on a controllable clock. Cover
an edge case with a short synthetic scenario through the same harness. ADR 0001
adds two levels above this: the built app through WebdriverIO, and a manual
checklist in a real match before each release.

## View rules (mandatory)

- **One component to a file**, named after it. A component another one renders
  gets a file of its own beside it.
- **A component takes its props apart in its signature**, one level into an object
  prop whose fields it reads. An object prop stays whole where it is optional, a
  union still to narrow, or handed on whole.
- **A handler of more than one statement is named**, a `handle…` const in the
  component's body.
- **A parameter of more than one field takes a named type**, and a union is of
  named types.
- **A function the linter finds too complex is split along what it does**, never
  by hiding a branch.

## Lint rules (mandatory)

- **`pnpm lint` is every check on the view**, reported in one run, and a push runs
  it: `pnpm install` points git at `.githooks/`.
- **ESLint's rules are a file to a concern under `eslint/`**, and
  `eslint.config.js` only lists them in order: a later block replaces a rule an
  earlier one set for the same file.
- **Dead code is `knip.jsonc`'s**: an export, a file or a dependency nothing reaches
  fails `pnpm lint`, and an ignore there says why.
- **A review finding a linter could have caught becomes a rule.** Where a maintained
  plugin checks it, turn the rule on in the same change and fix every place it flags.

## Comment rule (mandatory)

**A comment is at most three lines**, in every language in the repository, and
it says what is surprising rather than what is visible. A decision that needs a
paragraph goes in `docs/adr/`, with one line in the code pointing at it.

Before writing or editing a comment, read
[.claude/rules/comments.md](.claude/rules/comments.md).

## Commands

Run from the repository root; `make` lists them all.

```sh
make install   # install the view's dependencies
make dev       # run the app against the real League client
make dev-mock  # run the app against a fake Live Client Data API
make build     # build the release bundle
make format    # format the Rust and the view
make lint      # run every linter
make test      # run the tests
make check     # lint, test and build the view
make afk       # work through the ready-for-agent issues and merge them
```

**Run `make check` before every push, and do not push until it passes.** CI is off
([ADR 0009](docs/adr/0009-ci-is-off-while-the-gate-runs-locally.md)), and the
pre-push hook checks only the view. Never skip it with `--no-verify`.

**Do not package the app during development.** `make check` verifies a change, and
`make dev` or `make dev-mock` runs it. Build a release bundle or a DMG only when the
packaging itself is the work.

## Working here

- Non-trivial work starts from an issue, with scope settled there. Small fixes
  go straight to a pull request: a typo, an obvious bug, a dependency bump.
- Tests land with the behaviour they cover.
- A linter exception states its reason: `reason = "..."` in Rust, a `-- description`
  in ESLint.
- The code is the source of truth, then the docs, then a comment. A doc that
  contradicts the code is a bug in the doc: fix it in the change that found it.

## Agent skills

### Issue tracker

Issues live in GitHub Issues (kryptamine/lollipop), via the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
