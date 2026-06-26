---
purpose: Specify Indiana, the server that scans markdown markers and exposes agent-readable payloads.
max_lines: 50
status: draft
approval: pending
---

# INDIANA - PRD

> The server. Why: [PURPOSE.md](../PURPOSE.md). Where: [GOAL.md](../GOAL.md) / [PHASES.md](../PHASES.md). Markers: [IN_COMMANDS.md](IN_COMMANDS.md). MCP: [IN_MCP.md](IN_MCP.md). Daemon: [IN_DAEMON.md](IN_DAEMON.md). Invariants: [IN_PRINCIPLES.md](IN_PRINCIPLES.md).

## What it is
- A server you point at a repository.
- Scans all markdown files, fast, finding `::` commands.
- Aggregates them, generates an agent-ready payload, and exposes it through MCP.
- `indiana copy` renders the same payload for human clipboard fallback.
- One static binary, multi-mode: `indiana serve` (daemon), `indiana scan`, `indiana copy`, `indiana service install`.
- Initializes `.indiana/<command>/prompt.md` templates in each monitored root
  so users can tune compiled-prompt wording per repo ([IN_FOLDER.md](IN_FOLDER.md)).
- Faces: MCP, CLI, [Menulet](../MENULET_PRD.md) UI. All are clients; one core daemon serves all.
- Clients talk to the daemon over a Unix domain socket at `~/.indiana/indiana.sock`. Protocol: minimal JSON or bincode. No HTTP, local-only.

## The loop
1. Coding agent writes terse markdown ([Casablanca](../CASABLANCA_PRD.md)).
2. User reviews and tags lines: `::h ::l ::k ::f ::e ::q ::n ::a ::td`.
3. Indiana scans the repo, collects every marker + its context.
4. Indiana exposes the compiled payload through [IN_MCP.md](IN_MCP.md).
5. Agent reads the payload itself and acts.
- Clipboard copy stays for agents without MCP. No retyping.

## Scan engine
- How Indiana finds indianas, fast and read-only: [IN_SCAN.md](IN_SCAN.md).
- Markdown is source of truth; the index is a rebuildable view.

## Copy and counts
- Indiana owns the markers end to end: monitors, parses, compiles.
- It tallies per kind: actions, todos, notes, fixes, elaborates, questions, hate, love, keep.
- MCP and `indiana copy` return the same compiled payload.
- Payload = compiled prompts ([IN_COMMANDS.md](IN_COMMANDS.md)) + tagged context.
- Tracked indianas carry an ID so a directive can be followed across scans: [IN_IDENTITY.md](IN_IDENTITY.md).
- `::hate` contributes the canned explainer prompt — user never explains the why.
- MCP, CLI, and menulet only surface what Indiana computed; faces never count.

## Scope
- Resolved scope ([IN_SCOPE.md](IN_SCOPE.md)) travels into the payload so the agent sees exactly what was tagged.

## Out of scope
- Producing markers — humans tag; agents emit [Casablanca](../CASABLANCA_PRD.md) output.

## Decided
- Each indiana in the payload carries file path, line number, and ID so the agent can locate the target.
