---
status: draft
purpose: Capture Indiana's system shape and ownership boundaries.
approval: pending
---

# IN_ARCHITECTURE — system shape

## Core
- One daemon owns scan, index, scope, counts, compilation. Lifecycle: [IN_DAEMON.md](IN_DAEMON.md).
- Folder-local template structure: [IN_FOLDER.md](IN_FOLDER.md).
- Markdown stays source of truth. On-disk marker format: [IN_LINE.md](IN_LINE.md).
- Derived state is rebuildable. User config (monitored folders) is input, not derived ([IN_DAEMON.md](IN_DAEMON.md)).

## Faces
- MCP: agent reads structured payload directly.
- CLI: human scans, copies, operates service.
- Menulet: human watches and clicks.
- Faces never compute domain truth.

## Payload
- One compiled payload model feeds MCP and copy.
- MCP returns structure.
- Copy renders text.

## Links
- Server PRD: [IN_PRD.md](IN_PRD.md).
- MCP contract: [IN_MCP.md](IN_MCP.md).
- Daemon lifecycle: [IN_DAEMON.md](IN_DAEMON.md).
- Scan engine: [IN_SCAN.md](IN_SCAN.md).
- On-disk line format: [IN_LINE.md](IN_LINE.md).
- Invariants: [IN_PRINCIPLES.md](IN_PRINCIPLES.md).

## Chief of Staff todos (separate)
- A repo-local SQLite list at `.indiana/chief-of-staff/todos.db` ([IN_FOLDER.md](IN_FOLDER.md)), written and read by the `indiana todo` CLI for agents and humans.
- Not part of the marker index and not derived from source — a separate state store ([IN_PRINCIPLES.md](IN_PRINCIPLES.md) carve-out). It does not flow through the markdown write chokepoint.
