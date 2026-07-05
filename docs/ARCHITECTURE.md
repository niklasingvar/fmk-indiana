---
status: draft
purpose: System-level architecture — components, loops, state stores, boundaries. Engine internals live in indiana/IN_ARCHITECTURE.md.
approval: pending
---

# ARCHITECTURE — the system

> Concern map: [MENTAL_MODEL.md](../MENTAL_MODEL.md). Engine shape: [indiana/IN_ARCHITECTURE.md](indiana/IN_ARCHITECTURE.md). Invariants: [indiana/IN_PRINCIPLES.md](indiana/IN_PRINCIPLES.md).

## Components

| Component | Role | Runs as | Status |
|---|---|---|---|
| indiana core | scan, parse, scope, compile, write chokepoint | library (`crates/core`) | shipped |
| indiana daemon | monitors roots, serves state over Unix socket | `indiana serve` (`crates/indiana`) | shipped |
| CLI | human face: scan, copy, todo, frontmatter, templates | same binary | shipped |
| MCP | agent face: compiled payload as structured data | `indiana mcp` | built, unverified |
| menulet | glanceable face: folders, counts, one-click copy | Tauri app (`crates/menulet`) | shipped |
| context-model | per-repo memory agents read and write back | files in `.indiana/context-model/` | scaffold only |
| chief-of-staff | per-repo todos for humans and agents | `todos.db` + `indiana todo` | v1 shipped |
| casablanca | the editor: rich markdown editing, artifact review | Electron app (`crates/casablanca`) | prototype — MVP is ACTION_PLAN Phase 1 |

## The two loops

### Artifact loop (shipped)
1. Agent emits markdown into the repo.
2. Human tags lines with `::` markers.
3. Daemon scans, compiles markers into prompts + tagged context.
4. Agent reads the payload (MCP) or human pastes it (`indiana copy`).
5. Agent edits the artifact. Repeat.

### Knowledge loop (the differentiator, not yet real)
1. Every compiled prompt directs the agent to read `.indiana/context-model/` first.
2. Feedback markers (`::hate`, `::love`) instruct the agent to write the extracted rule back into `.indiana/context-model/`.
3. Next loop starts smarter. Feedback given once is never given twice.
- Contract is bidirectional: templates read from the context-model before, write to it after. Until prompts reference it, the context-model is dead weight.

## State stores

| Store | Location | Nature | Rebuildable |
|---|---|---|---|
| source markdown | user repo | the only truth | is the truth |
| marker index, counts, payload | daemon memory | derived cache | yes, rescan |
| monitored folders | `~/.indiana/config` | user input | no, config |
| copy cursor | `~/.indiana/copied.json` | interaction history | safe to delete |
| command templates | `.indiana/indianas/` | user-authored input | re-scaffold defaults |
| context-model | `.indiana/context-model/` | accumulated knowledge | no — this is the point |
| chief-of-staff todos | `.indiana/chief-of-staff/todos.db` | authoritative state | no |

## Boundaries
- Core computes; faces (CLI, MCP, menulet, casablanca) render. A face never parses, counts, or assembles prompts.
- One write chokepoint mutates user files: byte-preserving, atomic, mtime-guarded, idempotent.
- Indiana never runs an agent. It compiles; existing harnesses (Claude Code, Codex, Cursor) execute. Their tokens, their quota.
- casablanca is downstream of the agent, indiana is downstream of the human. They meet only in the folder.
- chief-of-staff state does not flow through the markdown chokepoint; `indiana todo` is its single face.

## Handoff evolution
1. Copy-paste — `indiana copy` to clipboard. Shipped.
2. MCP — agent pulls the payload itself. Built; verify end to end.
3. Auto-run — daemon dispatches markers as they appear, pausable. Not started; requires MCP proven first.
