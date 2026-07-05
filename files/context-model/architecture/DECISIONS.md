---
id: architecture.decisions
layer: architecture
status: active
owner: shared
purpose: The decision log — choices made, reversals, dead ends explored, and tombstones for deprecated files.
upstream: [architecture]
review_by: 2027-01-05
updated: 2026-07-05
---

# DECISIONS

- Format: `DATE — decision — reason — what we gave up`. Append-forward; never rewrite an entry, supersede it.

## Decisions

- 2026-07 — Casablanca IS the editor, self-built (Electron + Lexical); Nimbalyst vendored as reference only — borrowing patterns beats inheriting a codebase and its name — gave up: a head start on editor plumbing.
- 2026-07 — Visual support is a feature set inside Casablanca, not a separate module or product — one surface, one mental model — gave up: independent shipping of a viewer.
- 2026-07 — Command grammar is global; only prompt wording is per-repo (for now) — a stable grammar keeps the collector simple — gave up: per-repo command kinds, tracked as planned work in [ACTION_PLAN.md](../../../ACTION_PLAN.md).
- 2026-07 — This tree adopts the schema in [CONTEXT-MODEL.md](../CONTEXT-MODEL.md): frontmatter lifecycle, link-based dependencies, stability gradient — makes the tree machine-lintable and trust-at-query-time viable — gave up: freeform note-taking.

## Dead ends (do not re-explore without new facts)

- (none recorded yet — the first reverted experiment lands here with its post-mortem link)

## Tombstones (deprecated ids and their successors)

- (none yet — every `active → deprecated` transition adds a line here, per the lifecycle rules)
