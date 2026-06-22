---
status: draft
purpose: Sequence Indiana into usable shipping phases.
approval: pending
---

# PHASES

> Steps toward [GOAL.md](GOAL.md). Each phase ships something usable. Rome isn't built in a day.
> CLI-first: the server and its scan come before any UI. Menulet is a face added later.

## Phase 1 — Server up, CLI scans markdown
- Indiana daemon starts against a repo root. Lifecycle / socket / config: [INDIANA/IN_DAEMON.md](INDIANA/IN_DAEMON.md).
- Walk all markdown, fast; find `::` markers ([COMMANDS.md](INDIANA/IN_COMMANDS.md)).
- Iterate / list every marker from the CLI in one pass.
- Engine spec: [INDIANA/IN_SCAN.md](INDIANA/IN_SCAN.md).
- *No menulet, no copy bundle, no Casablanca, no meta model.*

## Phase 2 — Counts + watch
- Per-kind tallies (actions, notes, fixes, questions, hate, love, keep).
- Event-driven watch (FSEvents, ~300 ms debounce); full scan on startup.
- Markdown is source of truth; index rebuildable.

## Phase 3 — Copy bundle (CLI)
- Compile markers → generated prompts + tagged context.
- `indiana copy` → clipboard fallback for agents without MCP.
- Scope resolution, in order: inline + next-row first, section second, range last ([INDIANA/IN_SCOPE.md](INDIANA/IN_SCOPE.md)).

## Phase 4 — MCP payload
- Indiana exposes the compiled payload as a local MCP server.
- Agent reads pending markers, resolved scope, source paths, line numbers, IDs.
- Copy bundle remains a human fallback; MCP is the agent-native path.
- Interface spec: [INDIANA/IN_MCP.md](INDIANA/IN_MCP.md).

## Phase 5 — Menulet face
- macOS menulet in the menu bar.
- Add / remove monitored folders; list what's monitored.
- One-click Copy. Shows, never computes.

## Phase 6 — Casablanca in
- Agents emit Casablanca-formatted terse output.
- Indiana visualizes it; markers become first-class.

> Detail lives in each PRD: [IN_PRD.md](INDIANA/IN_PRD.md), [CASABLANCA_PRD.md](CASABLANCA_PRD.md).
