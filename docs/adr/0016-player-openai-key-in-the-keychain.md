---
status: accepted
---

# The player's OpenAI key lives in the Keychain

Advice calls OpenAI with the player's own key, straight from the app, with no server of ours in between. A key costs its owner money, so it is kept in the macOS Keychain, never in `settings.json` or a log, and it never returns to the view once saved.

- A key is stored only after `GET /v1/models/gpt-6-luna` answers 200. A rejected key or a network error leaves nothing stored, so a stored key was valid at least once.
- `app` reads the Keychain once at start and after a change, and hands the key to `core` as a value. `core` tests pass a fake key and the fake OpenAI's base URL.
- A debug build may take the key and base URL from the environment for `make dev-mock`. A release build ignores them.

## Considered options

- **A file with 0600 permissions:** no access prompt, but any process running as the player can read it.
- **Our own proxy holding one key:** no setup for the player, but we would pay for every game and run a server that handles their traffic.

## Consequences

- An ad-hoc build has a new signature every release ([ADR 0012](0012-ad-hoc-releases-until-the-apple-account-exists.md)), so macOS asks for Keychain access once after each update. Accepted until builds are notarized.
- The Advice Widget's switch cannot be turned on without a stored key, and removing the key turns it off.
- A 401 or a 429 `insufficient_quota` in a game stops OpenAI calls until a new key is saved. The Widget says why once, and the Advice tab marks the key.
