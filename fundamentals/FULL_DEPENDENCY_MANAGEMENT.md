---
status: draft
purpose: Relations — every dependency is a declared upstream edge pointing to a more stable layer, acyclic by construction.
approval: pending
---

# FULL DEPENDENCY MANAGEMENT

## Definition
- Every dependency between files is declared: normative `upstream` edges in frontmatter, reference links in the body.

## Rules
- Upstream edges point only to same-or-more-stable layers; the normative graph is acyclic by construction.
- If a file must not contradict another, that edge is written down — never implied.

## Test
- Dangling links in active files fail lint; an orphan justifies itself or is deprecated.

## Incorporation
- This repo: the graph rules in [docs/context-model/CONTEXT-MODEL.md](../docs/context-model/CONTEXT-MODEL.md) §5.
- System prompt: the system prompt routes reads by index and upstream ids — agents follow declared edges, never scan.
