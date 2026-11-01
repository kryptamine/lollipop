---
status: accepted
---

# The objective schedule is compiled into the app

The Objective schedule lives in code, not in a file downloaded like the Benchmark table, even though that channel already exists. Timing hints must work with no network and no table, so a downloaded file would still need a copy in the binary. That is two sources of truth for 1-2 changes a year, and a change in logic, such as the Elder rule, needs a release anyway.

Each entry names a `from_patch`, so a fix can ship before its patch. Hints run on any patch within a Season, and a new major `gameVersion` shows no hints until the app has an entry for it.

## Considered options

- **Confirming every patch:** closes the mid-season gap, but means frequent releases or a separate signed confirmation file.
- **Using the previous schedule on a new Season:** when 26.1 moved Baron from 25:00 to 20:00, it would have shown a hint at 24:15 for a Baron up for 5 minutes.

## Consequences

- A mid-season timer change shows a wrong hint until the fix arrives. 3 of the 6 changes in the research's history were mid-season. The owner reading patch notes and a collector check reduce this, and an undocumented change can take up to a week to notice.
- A fix only reaches a running app if it relaunches, so the app relaunches after `GameEnd` once an update is downloaded.
