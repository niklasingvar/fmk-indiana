---
status: draft
purpose: Time — draft → active → deprecated → archived; active is trusted at query time, nothing is deleted.
approval: pending
---

# FILE LIFE CYCLE MANAGEMENT

## Definition
- Every file is in exactly one state: draft → active → deprecated → archived. No other states, no skipping.

## Rules
- Active means trusted at query time — read as truth, never re-verified against sources.
- Nothing is deleted; archive is the only exit, git the only true eraser.

## Test
- A file that cannot be trusted must not be active — there is no third state.

## Incorporation
- This repo: the state machine in [docs/context-model/CONTEXT-MODEL.md](../docs/context-model/CONTEXT-MODEL.md) §4; `status` frontmatter on every build doc.
- System prompt: the system prompt's "treat `active` files as truth" line is this fundamental, verbatim.
