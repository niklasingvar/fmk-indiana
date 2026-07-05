---
status: draft
purpose: Vision for Chief of Staff — human/agent focus management including a ticket system. Deliberately not yet designed.
approval: pending
---

# COS — VISION

> Unbaked by decision (2026-07): vision first, architecture later. Nothing here is a spec; do not build against this file.

## What it is
- Focus management for a human working with agents — including a ticket system.
- Not status-tracking; attention-tracking. The question it answers: "what should I be doing right now?" — one glance.

## Direction
- Two queues: Human TODOs (decide, review, provide) and Agent TODOs (autonomous work items).
- Indiana executes against the agent queue; completed loops and open questions flow back as Human TODOs.
- Division of labor: the human queue stays short and decision-shaped; the agent queue drains in the background.
- The menulet becomes its glanceable surface — later ([MENULET_PRD.md](../menulet/MENULET_PRD.md) stays lightweight for now).

## What exists today (placeholder, not design)
- `.indiana/chief-of-staff/todos.db` — flat list via `indiana todo add|list|delete`, `--json` for agents.
- Scaffolded `actions.md`, `notes.md`, `focus.md`.
- Treat these as a stub that proved plumbing; the queue model above may replace them wholesale.

## Before any architecture
- Settle the vision: what is a ticket, what state moves it between queues, what does "focus" mean operationally.
- Decide the relationship between `::todo`/`::action` markers and the queues.
- Then, and only then, a COS_PRD.
