---
status: accepted
---

# Public Benchmark tables on a personal key

Benchmark tables are served from a public R2 bucket while the collector still runs on a Riot personal key. This overrides the consequence in ADR 0003 that public tables need a production key. Riot's policy is that a product built on a personal key stays private, and the owner has read that and accepts the risk: there is no plan to apply for a production key for now.

A token and a private GitHub Release were the price of staying inside that policy, and the owner prefers the simpler channel: no secret in the app, and no GitHub API between the app and its table.

## Consequences

- Riot may act against the key or the account. The collector would stop, and the app would keep its last table until it goes Stale.
- The bucket holds tables only. A public domain opens a whole bucket, so backups stay in their own.
- Collection stays on one routing value, as the other routing values need the production key.
- `r2.dev` is rate limited and meant for development. Moving to a custom domain changes `TABLE_CHANNEL_URL`, so every install needs a rebuild.
