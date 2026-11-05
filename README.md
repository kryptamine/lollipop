# Lollipop

A macOS overlay for League of Legends. It shows how your game compares, minute by
minute, to how similar games usually go: CS, CS pace, Gold earned and level against
the Benchmarks for your champion, role and tier.

It also shows Timing hints before objectives spawn, a Build that follows the items
your enemies hold, and Advice from a model, which it can read aloud. Advice uses your
own OpenAI key, kept in the macOS Keychain.

Lollipop reads only Riot's public and local APIs: Live Client Data, the League client
API and the Riot web API. It never reads game memory or injects into the game
([ADR 0008](docs/adr/0008-public-and-local-riot-apis-only.md)).

## Install

Download the DMG from [Releases](https://github.com/kryptamine/lollipop/releases) and
copy the app to Applications. Builds are ad-hoc signed for now
([ADR 0012](docs/adr/0012-ad-hoc-releases-until-the-apple-account-exists.md)), so clear
the quarantine once:

```sh
xattr -dr com.apple.quarantine /Applications/Lollipop.app
```

Updates then install on their own, between games.

## Develop

You need Rust, Node 24 and pnpm. From the repository root:

```sh
make install   # install the view's dependencies
make dev-mock  # run the app against a fake Live Client Data API
make dev       # run the app against the real League client
make check     # lint, test and build the view; run before every push
```

`make` lists every command.

## Layout

| Path               | What it holds                                                   |
| ------------------ | --------------------------------------------------------------- |
| `crates/core`      | Domain and API polling, with no Tauri                           |
| `crates/app`       | The Tauri shell                                                 |
| `crates/collector` | The CLI that collects ranked matches and builds Benchmark tables |
| `deploy/`          | Ansible and systemd for the collector's VPS                     |
| `ui/`              | The Preact view                                                 |

## Docs

- [GLOSSARY.md](GLOSSARY.md): the terms used across code and docs.
- [docs/adr/](docs/adr/): the decisions and why they were made.
- [docs/spec/](docs/spec/): what the app does, in detail.
- [deploy/README.md](deploy/README.md): running the collector.
- [AGENTS.md](AGENTS.md): the rules for contributing, for people and agents alike.
