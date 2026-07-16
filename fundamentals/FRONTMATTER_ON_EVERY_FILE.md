---
status: draft
purpose: The contract carrier — id, layer, status, owner, upstream. No frontmatter, no trust.
approval: pending
---

# FRONTMATTER ON EVERY FILE

## Definition
- YAML frontmatter is the machine-readable contract: id, layer, status, owner, purpose, upstream, review date.

## Rules
- A file without frontmatter does not exist — agents skip it, lint flags it, it earns no trust.
- What is not in frontmatter is not enforced.

## Test
- Lifecycle and dependencies are checkable from frontmatter alone.

## Incorporation
- This repo: the frontmatter contract in [docs/context-model/CONTEXT-MODEL.md](../docs/context-model/CONTEXT-MODEL.md) §3; `indiana frontmatter --write` lints and seeds it ([docs/indiana/IN_FOLDER.md](../docs/indiana/IN_FOLDER.md)).
- System prompt: prompt templates carry their own frontmatter contract (`command`, `command_type`); invalid frontmatter falls back to embedded defaults.
