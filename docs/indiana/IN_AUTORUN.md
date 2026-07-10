---
purpose: Specify auto-run — the daemon dispatching `-a` markers to an agent over ACP.
max_lines: 70
status: draft
approval: pending
---

# IN_AUTORUN — auto-run dispatch

> The one place the daemon runs an agent. Grammar: [IN_COMMANDS.md](IN_COMMANDS.md). On-disk state: [IN_LINE.md](IN_LINE.md). Identity: [IN_IDENTITY.md](IN_IDENTITY.md). Engine: [IN_SCAN.md](IN_SCAN.md). System boundary: [../ARCHITECTURE.md](../ARCHITECTURE.md).

## Stance
- The core never runs an agent; it compiles ([../ARCHITECTURE.md](../ARCHITECTURE.md)). Auto-run is the deliberate, marker-scoped exception, and it lives in the daemon — a face — not the core.
- Opt-in twice: per marker via the `-a` flag, and globally via `config.auto_run` (default off — the "pausable" switch). Neither alone dispatches.
- Applies only to directives that act directly: `::fix`, `::elaborate`, `::prompt`. The gated `::delete` is excluded — auto-run and confirm-first conflict.

## Lifecycle
- `::fix -a banana` typed and saved → the daemon's scan sees the `-a` flag.
- **Claim**: the write chokepoint rewrites the line to `::fix[happy-otter:working] banana` — mint an id, set `working`, strip the `-a` flag — atomic, mtime-guarded, idempotent ([IN_LINE.md](IN_LINE.md)). The own-write is suppressed so the claim does not re-trigger a scan.
- **Dispatch**: compile the single marker (same prompt a paste would carry) plus a coda telling the agent to delete the marker line and commit; run one ACP turn over it.
- **Resolve**: the agent applies the change, removes the marker line, and commits. On the next scan the marker is simply gone.
- **Fail**: if the turn ends with the `working` marker still present, the daemon rewrites it to `failed` — visible, and not re-dispatched.

## Transport — ACP
- The daemon is an ACP *client*; the agent adapter (default `npx -y @zed-industries/claude-code-acp`, fetched/cached on first use — needs Node, [../DISTRO.md](../DISTRO.md)) is the *agent*. Newline-delimited JSON-RPC over the child's stdio — hand-rolled, no async runtime, same stance as [IN_MCP.md](IN_MCP.md).
- One turn: `initialize` → `session/new { cwd = owning root }` → `session/prompt`. Streamed `session/update` events are logged to `~/.indiana/dispatch/<id>.log`.
- **Full autonomy**: `session/request_permission` is auto-granted (allow-always preferred), and `fs/*` requests are served against the working tree, so edits and `git` run with no human. A per-repo policy is a later refinement.
- Adapter resolution mirrors the other binaries: standard locations then PATH, overridable by `config.agent.command` (plus `args`, `env`).

## Guarantees
- Re-dispatch is prevented three ways: the `working`/`done`/`failed` status (not a fresh candidate), an in-memory in-flight set keyed by marker id, and own-write suppression.
- Concurrency is capped (`MAX_INFLIGHT`); excess candidates wait for a later rebuild cycle.
- Completion is decided by inspecting the file, not the stop reason: the agent resolves a marker by deleting its line, so a surviving `working` marker is a failure however the turn ended.
- Marker mutation still flows through the one write chokepoint ([IN_PRINCIPLES.md](IN_PRINCIPLES.md)); the id + `working`/`done`/`failed` marks are its only auto-run writes. The agent's own edits are outside that door, as any agent's always are.

## Config
- `config.auto_run: bool` — global kill-switch, default off. Reloaded each cycle, so toggling it needs no restart.
- `config.agent: { command, args, env }` — how to launch the ACP adapter. Default `npx -y @zed-industries/claude-code-acp`; set `command` to an installed bin with empty `args` to skip npx.

## Recovery
- A `working` marker with no live turn (daemon restarted mid-run, or a prior cycle crashed) is re-dispatched best-effort on the next scan.

## Decided
- The `working` state is written to source, not held in memory — it survives restart, is visible to the user, and needs no separate store.
- Success = the agent removed the line; the daemon does not itself write `done` on the happy path (it would race the agent's deletion). It writes `failed` only when the marker survives.
