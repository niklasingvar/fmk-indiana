---
status: draft
purpose: Every fact has exactly one home; everywhere else it is a link, never a copy.
approval: pending
---

# SINGLE SOURCE OF TRUTH

## Definition
- Every fact has exactly one home file; every other appearance is a link, never a copy.

## Rules
- To change a fact, edit its home.
- Readability may restate in at most one clause, with the link attached; the restatement is non-normative.

## Test
- The same fact found in two files is a violation — the more stable file keeps it, the other links.

## Incorporation
- This repo: the SSOT routing table and lint sweep in [docs/context-model/CONTEXT-MODEL.md](../docs/context-model/CONTEXT-MODEL.md) §6, §9; one canonical name per concept in [MENTAL_MODEL.md](../MENTAL_MODEL.md).
- System prompt: the context-model seed (`crates/core/templates/context-model/CONTEXT-MODEL.md`) carries the routing table into every monitored repo; every loop reads it unconditionally.
