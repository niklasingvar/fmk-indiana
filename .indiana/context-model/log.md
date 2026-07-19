---
id: log
layer: journal
status: active
owner: agent
purpose: Append-only record of every loop that touched this repo — greppable, chronological, never edited.
upstream: []
updated: 2026-07-06
---

# LOG

- Entry format: `## [YYYY-MM-DD] <command> | <target> — one-line outcome`. Append only; never edit or delete an entry.
- Greppable by design: `grep "^## \[" log.md | tail -5` shows the last five loops.

## [2026-07-05] init | context-model — tree scaffolded per CONTEXT-MODEL.md schema; seed files created, all indexed.
## [2026-07-06] adopt | .indiana/context-model — prepared tree from files/context-model installed as the dogfood instance; INDIANA.md folded into learnings/INBOX.md; stale montmartre folder removed.
## [2026-07-19] fix | fundamentals/MARKDOWN_AS_CODE.md — grounded definition in docs-as-code best practice (git + review + CI gates; markdownlint/Vale/lychee); marker removed.
## [2026-07-19] fix | test.md — four paragraphs on Indiana test strategy (spec↔test map, fixtures, layers, non-goals); marker removed.
