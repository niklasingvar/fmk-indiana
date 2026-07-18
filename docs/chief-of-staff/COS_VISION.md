---
status: draft
purpose: Vision for Chief of Staff — human/agent focus management. The first baked slice is the task tracker and action log (COS_PRD.md).
approval: pending
---

# COS — VISION

> The shipped slice is specced in [COS_PRD.md](COS_PRD.md); this file holds the direction beyond it.

## What it is
- Focus management for a human working with agents — including a ticket system.
- Not status-tracking; attention-tracking. The question it answers: "what should I be doing right now?" — one glance.

## Decided (2026-07)
- Two queues, fed by markers: `::todo` / `::task` → Agent queue, `::action` → Human queue.
- The tracker is a hand-editable markdown file (`.indiana/chief-of-staff/tasks.md`); the action log (`log.md`) records what ran. Contract: [COS_PRD.md](COS_PRD.md).
- A captured marker gets an injected id and a tracker line with an origin backlink; the marker line in source stays the single truth.
- `::action` compiles as a human-queue item: agents never execute it.
- `todos.db` and `indiana todo` retired; `indiana task` / `indiana log` are the CLI face.

## Direction
- Indiana executes against the agent queue; completed loops and open questions flow back as Human TODOs.
- Division of labor: the human queue stays short and decision-shaped; the agent queue drains in the background.
- The menulet becomes its glanceable surface — later ([MENULET_PRD.md](../menulet/MENULET_PRD.md) stays lightweight for now).

## Non-goals (for now)
- Dispatching a task from the tracker or the Casablanca panel (v1 gesture: jump to origin).
- A menulet focus view.
- What "focus" means operationally beyond the queues — still to settle before more architecture.
