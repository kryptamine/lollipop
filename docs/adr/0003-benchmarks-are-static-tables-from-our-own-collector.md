---
status: accepted
---

# Benchmarks are static tables from our own collector

Benchmarks are built offline by a Rust collector on a VPS and shipped as one versioned file that the app downloads. The app has no backend and no database. A table is never swapped during a game, so a Marker cannot change meaning mid-match.

## Considered options

- **A public dataset or stats site:** the ones checked are too small, limited to high tiers, snapshot-only or years old, and a static dataset goes stale within a patch. Their licences do not clear Riot's terms, and LoLalytics' terms could not be read, so no site was scraped.
- **A patch-diff window:** rejected for the last two patches, so data exists on patch day. A buffed champion's Benchmark lags by about a patch, and reworked champions are listed by hand.
- **An all-tier fallback for thin Cells:** a Master player on a rare pick would always look ahead, so fallback stops at adjacent tiers.

## Consequences

- A table built on a personal key may not be public under Riot's policy. [ADR 0011](0011-public-tables-on-a-personal-key.md) accepts that risk and serves tables publicly anyway.
- The VPS holds the manifest signing key, so signing protects against a compromised host, not a compromised VPS.
