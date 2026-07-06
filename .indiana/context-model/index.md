---
id: index
layer: journal
status: active
owner: agent
purpose: The routing file — one line per file in this tree; read after the schema, before everything else.
upstream: []
updated: 2026-07-05
---

# INDEX

- Line format: `[id] path (status) — purpose`. One line per file, identical to the file's `purpose` field. Nothing else lives here.

## meta
- [context-model] CONTEXT-MODEL.md (active) — The schema of the context-model — how this tree is structured, read, written, linked, and kept alive.

## purpose
- [purpose] purpose/PURPOSE.md (active) — Why this project exists, who it serves, and what it will never be — the tree's compression of the repo vision.
- [purpose.glossary] purpose/GLOSSARY.md (active) — The ubiquitous language — one term, one meaning, used identically in code, docs, and UI.

## architecture
- [architecture] architecture/ARCHITECTURE.md (active) — The system's shape, boundaries, and the invariants no loop may break.
- [architecture.decisions] architecture/DECISIONS.md (active) — The decision log — choices made, reversals, dead ends explored, and tombstones for deprecated files.

## rules
- [rules.global] rules/GLOBAL.md (active) — Cross-cutting rules for how every artifact in this repo must be created, regardless of type.
- [rules.presentations] rules/presentations/RULES.md (draft) — How presentation artifacts must be made in this repo — templates, structure, and iteration order.

## preferences
- [preferences.operator] preferences/OPERATOR.md (active) — The operator's confirmed taste — softer than rules, harder than hunches; read before every diagnostic loop.

## learnings
- [learnings.inbox] learnings/INBOX.md (active) — The single write-first target for every fresh insight from a loop — atomic entries awaiting consolidation.

## journal
- [index] index.md (active) — The routing file — one line per file in this tree; read after the schema, before everything else.
- [log] log.md (active) — Append-only record of every loop that touched this repo — greppable, chronological, never edited.
