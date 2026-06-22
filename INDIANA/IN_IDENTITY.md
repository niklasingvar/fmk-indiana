---
purpose: Specify which indianas get a tracked identity, and why identity lives in the source.
max_lines: 50
status: draft
approval: pending
---

# IN_IDENTITY — tracked vs ephemeral

> Some indianas need to be followed across scans; most do not. Markers: [IN_COMMANDS.md](IN_COMMANDS.md). Engine: [IN_SCAN.md](IN_SCAN.md).

## Two kinds
- Ephemeral: read-only, no ID. Compiled into the bundle and forgotten. The default.
- Tracked: carries an ID so the same indiana is recognized scan after scan, even if it moves.

## Why selective
- Most reactions are fire-and-forget — the human tags, copies, moves on.
- A few tasks must be followed until resolved; an ID lets Indiana keep pointing at the same one.
- IDs everywhere would be noise in the source and churn in the index.

## Which get IDs
- Tracked: `::action` / `::todo` only — the user tasks that carry `done` / `failed` and get checked off in the menulet.
- Ephemeral: everything else — reactions, `::fix`, `::elaborate`, `::question`, `::note`. Compiled and forgotten.
- No manual promotion. The grammar decides; one rule, nothing to configure per line.

## Identity lives in the source
- A tracked indiana's ID is written into its line, once, on first sight. Exact on-disk syntax: [IN_LINE.md](IN_LINE.md).
- The ID is the persistence — no database. Delete the index, rescan, identity returns from source.
- The ID travels with the line: move it to another file, the ID goes with it.
- Format: two generated pronounceable syllable tokens, lowercase — pattern `[a-z]+-[a-z]+(-[0-9]+)?` (e.g. `frata-nimta`, `lurvo-pannik`).
- Generated, not drawn from a word list: no bundled dictionary, effectively unlimited pool.
- On collision, append a counter: `-2`, `-3`. IDs never reused within a scan.
- Completion state lives in source too, on `::action` / `::todo` only — the user tasks worth tracking to closure.
- An action or todo may be marked `done` (and `failed`); the state is written into its line, beside the ID.
- Why in source: the menulet checks an item off — or an agent marks it over MCP ([IN_MCP.md](IN_MCP.md)) — and the mark persists, surviving index deletion like the ID.
- Reactions and other directives stay stateless — the review loop is ephemeral, not a tracker.

## Write discipline
- Injection is the only write Indiana makes ([IN_SCAN.md](IN_SCAN.md): read-only otherwise).
- Atomic: write temp, fsync, rename. Guard on mtime — if the file changed under us, abort and re-queue.
- Idempotent: a second pass on an already-tagged line produces identical bytes.

## Decided
- Generated pronounceable tokens. Readable, memorable, no dictionary to bundle or drift — unlimited pool by construction.
- Ephemeral indianas get no handle. A handle is state outside source; the bundle addresses them by position.

## Repair (D7)
- Malformed brackets are repaired, not trusted. An id failing `[a-z]+-[a-z]+(-[0-9]+)?` gets a fresh id; an unknown status word is dropped to open. Must stay idempotent (a repaired line rescans clean).
