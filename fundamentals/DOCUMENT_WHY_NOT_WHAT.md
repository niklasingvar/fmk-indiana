---
status: draft
purpose: Write what the diff cannot say; never mirror what a grep already knows.
approval: pending
---

# DOCUMENT WHY, NOT WHAT

## Definition
- Docs record intent, trade-offs, and traps — what the diff cannot say.

## Rules
- Never mirror what a grep or file-read already reveals; code is the truth for what code does.
- `::hate` writes the diagnosis ("operator hates X because Y"), never just the symptom.

## Test
- A file that restates one grep has negative value — deprecate on sight.

## Incorporation
- This repo: the anti-mirror rule in [docs/context-model/CONTEXT-MODEL.md](../docs/context-model/CONTEXT-MODEL.md) §5 and the writing rules in [docs/AGENT_WRITING.md](../docs/AGENT_WRITING.md).
- System prompt: diagnostic command templates (`hate`, `love`, `note`) instruct the agent to write the why into the tree, not the what.
