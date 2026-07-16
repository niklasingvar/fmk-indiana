---
status: draft
purpose: A folder is a mission — artifact, context, and configuration travel together; no state outside it.
approval: pending
---

# FOLDER IS THE UNIT OF WORK

## Definition
- A folder is a mission: it holds the artifact, the context, and the configuration for how AI behaves there.

## Rules
- Everything an agent needs lives inside the folder, under `.indiana/`.
- No state outside the folder; declared carve-outs live in [docs/indiana/IN_PRINCIPLES.md](../docs/indiana/IN_PRINCIPLES.md).

## Test
- Point any agent at the folder and it has everything; move the folder and nothing is lost.

## Incorporation
- This repo: `.indiana/` is a dogfood instance like any monitored repo ([MENTAL_MODEL.md](../MENTAL_MODEL.md)).
- System prompt: the system prompt addresses everything relative to the repo root — the folder is the whole world it names.
- settings.json: lives inside the folder (`.indiana/casablanca/settings.json`) — configuration travels with the mission.
