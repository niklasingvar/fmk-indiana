---
status: draft
purpose: The artifact is the interface; feedback lives in the file as `::` markers, not in a conversation.
approval: pending
---

# CONTENT OVER CHAT

## Definition
- The artifact is the interface; work and feedback happen in the file.

## Rules
- Feedback is written into the artifact as `::` markers, never described in a conversation.
- Every round trip through chat is overhead to collapse.

## Test
- If giving feedback requires explaining where in a chat window, this is broken.

## Incorporation
- This repo: dogfood — `::` markers in our own docs, compiled by `indiana copy`/MCP ([docs/PURPOSE.md](../docs/PURPOSE.md)).
- System prompt: `crates/core/templates/system_prompt.md` (instance: `.indiana/SYSTEM_PROMPT.md`) instructs every agent to act on markers in place, not to open a dialogue.
