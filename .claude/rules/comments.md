---
description: How to write comments, and what never to comment
---

# Comments

These rules cover every kind of comment, in Rust (`//`, `///`, `//!`), HTML and
JavaScript alike.

## Rules

- Comment only a declaration or block whose intent is not obvious at a glance.
  Obvious code gets none, public or not; a comment that only expands the
  identifier's name is worse than silence.
- **A comment is at most three lines.** A hard cap. A longer explanation belongs
  in `docs/adr/`, with one line in the code pointing at it.
- Explain the gist of the thing the comment sits on: what it does, plus the why
  when that is not obvious. Call sites move, so leave out who calls it and when.
- A measured value is worth stating. Its provenance (which probe, issue, session
  or commit) lives in `git log` and `docs/adr/`.
- State a contract where it is relied on, rather than where a caller might one
  day lean on it.
- Keep a comment that stops a reader "fixing" the code and breaking it: a
  measured constant, a constraint the Riot API imposes, an ordering that matters,
  a case that looks unhandled and is not.
- Plain words. If a sentence needs a second read, rewrite it shorter.
- An `#[allow(...)]` says why in the same line, or uses `reason = "..."`.
- In tests, a comment says what the case protects: the behaviour that would
  otherwise be lost, not what the assertion does.
- A field, method or constant deleted takes its comment with it.

## Examples

Bad, restates the code and names the caller:

```rust
// Called by poll_live_client on every tick.
// Parses the snapshot from the body.
pub fn parse_snapshot(body: &str) -> Option<Snapshot> { ... }
```

Good, what the signature cannot state:

```rust
/// Returns `None` when the payload isn't a running game the player is part of.
pub fn parse_snapshot(body: &str) -> Option<Snapshot> { ... }
```

Good, stops a reader "fixing" it:

```rust
// The League client serves loopback HTTPS with a self-signed certificate,
// and this client only ever talks to the fixed loopback URL above.
.danger_accept_invalid_certs(true)
```

Good, none at all:

```rust
fn body(game_time: f64) -> String { ... }
```

## Comment pass

Write the code first. As the last step before reporting, check the comments this
change added or touched against the rules above, editing comments only.
