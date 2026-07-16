---
status: draft
purpose: Never run an own agent or own the token bill; any harness connects to the folder.
approval: pending
---

# HARNESS AGNOSTIC

## Definition
- The system never runs its own agent; compiled markers are handed to the harness the user already has.

## Rules
- Their tokens, their quota, their harness — never own the token bill.
- Every handoff phase stays harness-neutral: copy-paste, MCP, auto-run ([VISION.md](../VISION.md)).

## Test
- Swap Claude Code for Codex or Cursor and nothing in the folder changes.

## Incorporation
- This repo: MCP and ACP are faces over the same compiled payload as `indiana copy` ([docs/indiana/IN_PRINCIPLES.md](../docs/indiana/IN_PRINCIPLES.md)).
- System prompt: the system prompt is plain text that works pasted into any agent — it assumes no harness.
- settings.json: `model` selects an ACP model for a turn; it never selects a bundled agent.
