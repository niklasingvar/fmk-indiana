---
status: draft
purpose: Specify Indiana as a local MCP server for agent-readable marker payloads.
max_lines: 50
approval: pending
---

# IN_MCP — agent payload interface

> Indiana's agent-native face. Server: [IN_PRD.md](IN_PRD.md). Markers: [IN_COMMANDS.md](IN_COMMANDS.md). Scope: [IN_SCOPE.md](IN_SCOPE.md). Identity: [IN_IDENTITY.md](IN_IDENTITY.md). Invariants: [IN_PRINCIPLES.md](IN_PRINCIPLES.md).

## Intent
- Agent should read the review payload itself.
- Human copy-paste is fallback, not the primary loop.
- MCP exposes what Indiana already compiled. It does not parse, count, or resolve scope.

## Contract
- Local-only MCP server, backed by the Indiana daemon.
- One repo root per monitored workspace.
- Agent can list pending indianas.
- Agent can read one compiled indiana by ID.
- Agent can read the full compiled payload.
- Agent can ask for marker grammar and prompt meanings.

## Payload shape
- Marker ID.
- Marker kind and raw token.
- Compiled prompt.
- User message, when present.
- Source file path.
- Source line number.
- Resolved scope kind.
- Resolved scope content.
- Status for user tasks: pending, done, failed.

## Boundaries
- MCP never edits user files directly.
- MCP never invents marker meaning.
- MCP never returns stale state without saying scan status.
- Completion writes, if any, still flow through Indiana's single write chokepoint ([IN_PRINCIPLES.md](IN_PRINCIPLES.md)).

## Decided
- MCP and `indiana copy` share one compiled payload model.
- Copy formatting is a renderer. MCP output is structured payload.
- Agent-native means no clipboard dependency when the agent supports MCP.
