---
purpose: Specify auto-run — the daemon dispatching `-a` markers to an agent over ACP.
max_lines: 70
status: draft
approval: pending
---

# IN_AUTORUN — auto-run dispatch

> The one place the daemon runs an agent. Code ownership map: [IN_AUTORUN_ARCHITECTURE.md](IN_AUTORUN_ARCHITECTURE.md). Grammar: [IN_COMMANDS.md](IN_COMMANDS.md). On-disk state: [IN_LINE.md](IN_LINE.md). Identity: [IN_IDENTITY.md](IN_IDENTITY.md). Engine: [IN_SCAN.md](IN_SCAN.md). System boundary: [../ARCHITECTURE.md](../ARCHITECTURE.md).

## Stance
- The core never runs an agent; it compiles ([../ARCHITECTURE.md](../ARCHITECTURE.md)). Auto-run is the deliberate, marker-scoped exception, and it lives in the daemon — a face — not the core.
- Opt-in twice: per marker via the `-a` flag, and per repo via `autoRun` in that repo's `.indiana/casablanca/settings.json` (the daemon reads it). Per-repo `true`/`false` wins; unset falls back to the global `config.auto_run` default (off). Neither the flag alone nor an un-opted repo dispatches.
- Applies only to directives that act directly: `::fix`, `::elaborate`, `::prompt`. The gated `::delete` is excluded — auto-run and confirm-first conflict.

## Lifecycle
- `::fix -a banana` typed and saved → the daemon's scan sees the `-a` flag.
- **Claim**: the write chokepoint rewrites the line to `::fix[happy-otter:working] -a banana` — mint an id, set `working`; flags and message pass through byte-for-byte ([IN_LINE.md](IN_LINE.md): flags stay in source; the bracket status gates dispatch). Atomic, mtime-guarded, idempotent. The own-write is suppressed so the claim does not re-trigger a scan.
- **Dispatch**: compile the single marker plus the same versioned system prompt a paste carries (`.indiana/SYSTEM_PROMPT.md`, [IN_FOLDER.md](IN_FOLDER.md)) plus a coda telling the agent to delete the marker line and commit; run one ACP turn over it. By construction, auto-run and `indiana copy` instantiate agents with the same system prompt.
- **Resolve**: the agent applies the change, removes the marker line, and commits. On the next scan the marker is simply gone.
- **Fail**: if the turn ends with the `working` marker still present, the daemon rewrites it to `failed` — visible, and not re-dispatched.

## Manual numeric groups
- `::<cmd> -<number> [message]` assigns a marker to a repo-wide manual batch. It does not auto-run on save and does not depend on `config.auto_run`.
- The daemon reports each group and its member count. A menulet Run claims every open/failed member, compiles only that group, and sends one ACP turn; Copy returns the same grouped payload without mutation.
- Success requires every claimed marker line to be removed. Any surviving `working` members become `failed`.
- One group run produces one commit. Group labels stay in source while working/failed so the batch remains visible and retryable.

## Transport — ACP
- The daemon is an ACP *client*; the agent adapter (default `npx -y @zed-industries/claude-code-acp`, fetched/cached on first use — needs Node, [../DISTRO.md](../DISTRO.md)) is the *agent*. Newline-delimited JSON-RPC over the child's stdio — hand-rolled, no async runtime, same stance as [IN_MCP.md](IN_MCP.md).
- One turn: `initialize` → `session/new { cwd = owning root }` → `session/prompt`. Streamed `session/update` events are logged to `~/.indiana/dispatch/<id>.log` and projected into an in-memory per-job transcript, served to faces via the `jobtranscript` socket command (since-seq polling; dies with the job — [../casablanca/CASABLANCA_AGENT_JOBS.md](../casablanca/CASABLANCA_AGENT_JOBS.md)).
- **Full autonomy**: `session/request_permission` is auto-granted (allow-always preferred), and `fs/*` requests are served against the working tree, so edits and `git` run with no human. A per-repo policy is a later refinement.
- Adapter resolution mirrors the other binaries: standard locations then PATH, overridable by `config.agent.command` (plus `args`, `env`).

## Agent questions
- A turn may pause on ACP form `elicitation/create`; this is a human decision, distinct from the auto-granted permission request.
- The daemon owns the live turn and exposes it to faces as `running` or `awaiting_input`. A face answers the pending request, then the same turn resumes. Questions and answers also land in the job's transcript, so a follow view reads as a conversation.
- First UI contract: one string field per question. Unsupported schemas and URL-mode requests return an ACP error rather than silently collecting sensitive data.
- Jobs are daemon memory, not marker state. Restarting the daemon stops their child processes; the normal surviving `working` marker recovery path applies.

## Guarantees
- **One turn per repo at a time.** A repo with a live agent turn (`-a` or group) dispatches nothing else until it ends; concurrent agents in one working tree race each other's edits and sweep each other's files into commits. Waiting candidates stay unclaimed and dispatch when the turn finishes (the worker re-checks the repo directly, so a failed turn that touched no files cannot strand them). Different repos run in parallel.
- Re-dispatch is prevented three ways: the `working`/`done`/`failed` status (not a fresh candidate), an in-memory in-flight set keyed by marker id, and own-write suppression.
- Manual group runs use a repo-path + number in-flight key, so repeated clicks cannot launch the same batch twice.
- Concurrency is capped (`MAX_INFLIGHT`); excess candidates wait for a later rebuild cycle.
- Completion is decided by inspecting the file, not the stop reason: the agent resolves a marker by deleting its line, so a surviving `working` marker is a failure however the turn ended.
- Marker mutation still flows through the one write chokepoint ([IN_PRINCIPLES.md](IN_PRINCIPLES.md)); the id + `working`/`done`/`failed` marks are its only auto-run writes. The agent's own edits are outside that door, as any agent's always are.

## Config
- Per-repo opt-in: `autoRun: true` in `<repo>/.indiana/casablanca/settings.json` — committable, travels with the repo, set by `indiana casablanca set autoRun true` or by opening the repo in Casablanca. This is the primary control.
- Per-repo model: optional `model: "sonnet"` in the same file, set by `indiana casablanca set model sonnet`. The daemon selects that value through ACP session config before prompting; unset leaves the adapter default, and unsupported values fail the turn rather than silently using another model.
- `config.auto_run: bool` (`~/.indiana/config.json`) — the global default when a repo hasn't set `autoRun`; default off. Reloaded each cycle.
- `config.agent: { command, args, env }` — how to launch the ACP adapter (machine-level, stays global). Default `npx -y @zed-industries/claude-code-acp`; set `command` to an installed bin with empty `args` to skip npx.

## Recovery
- A `working` marker with no live turn (daemon restarted mid-run, or a prior cycle crashed) is re-dispatched best-effort on the next scan.

## Decided
- The `working` state is written to source, not held in memory — it survives restart, is visible to the user, and needs no separate store.
- Success = the agent removed the line; the daemon does not itself write `done` on the happy path (it would race the agent's deletion). It writes `failed` only when the marker survives.
- Numeric groups are manual orchestration, not a second spelling of `-a`. `-a` remains per-marker; group Run is one explicit turn for all members.
