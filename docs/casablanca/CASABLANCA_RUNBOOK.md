---
status: draft
purpose: Break M13 Casablanca into a separate implementation plan.
approval: pending
---

# CASABLANCA_RUNBOOK

## Boundaries
- Casablanca is not Indiana.
- Casablanca does not own, semantically parse, or mutate `::` markers; it may apply presentation-only highlighting to recognized marker text.
- Casablanca formats agent output going to the human.
- Indiana compiles human markers going to the agent.

## M13.1 — Output Grammar
- Define the smallest terse agent-output grammar.
- Start with sections: `changed`, `verified`, `risk`, `next`.
- Keep it text-first so any coding agent can emit it.
- Verify: examples in this folder parse as fixtures.

## M13.2 — Renderer Contract
- Decide renderer target: markdown first, app view later.
- Define one structured model independent of renderer.
- Verify: one fixture renders to markdown byte-for-byte.

## M13.3 — Templates
- Store prompt/output templates as data.
- No agent-output wording hardcoded in renderer logic.
- Verify: changing template wording does not touch parser or renderer code.

## M13.4 — Indiana Relationship
- Link, do not couple.
- Shared repo conventions are allowed.
- Shared runtime or shared socket is not allowed until a real use case appears.
- Verify: Casablanca tests do not import Indiana crates.

## Deferred
- Visual app.
- Agent SDK integration.
- Shared timeline with Indiana.
- Semantic marker-aware features.
