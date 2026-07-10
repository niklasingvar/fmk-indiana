# Auto-run: `::fix -a` dispatches Claude Code over ACP on save

## Context

Today Indiana **compiles** markers and hands them off — a human hits `Copy all`, pastes into
an agent, the agent fixes. `docs/ARCHITECTURE.md:54` states the invariant: *"Indiana never runs
an agent."* The roadmap's third handoff step (`ARCHITECTURE.md`, `ACTION_PLAN.md` Phase 8) is
**Auto-run — daemon dispatches markers as they appear**, explicitly "not started."

This change ships the first slice of that step. The desired loop:

1. User types `::fix -a banana` in any watched file and saves.
2. The daemon sees the `-a` (auto) flag, **claims** the marker by rewriting the line to
   `::fix[happy-otter:working] banana` (id minted, `working` status, `-a` consumed).
3. The daemon spawns a Claude Code **ACP agent** subprocess and runs one turn with the compiled
   prompt.
4. The agent applies the fix, **removes the marker line**, and commits.

**Transport is Zed's Agent Client Protocol (ACP)** — JSON-RPC 2.0 over stdio between a *client*
(Indiana's daemon) and an *agent* (Claude Code, via its ACP adapter). This is the same protocol
Zed uses to drive agents, and the vendored nimbalyst tree already speaks it to Codex
(`@agentclientprotocol/sdk`, `@zed-industries/codex-acp`). We follow that pattern but implement the
client natively in Rust.

This is a deliberate crossing of the "never runs an agent" line: the **daemon** (a face/runtime,
not the pure core) may now run an agent when a marker opts in with `-a`. The core stays a compiler;
the ACP client is new daemon-side code. Commit correctness itself is being sorted separately
(`docs/AGENT_COMMIT.md`); here we only instruct the agent to commit.

**Decisions locked with the user:** daemon dispatches (works in any editor) · **ACP is the
transport** · on-disk form is `[id:working]` reusing the existing bracket · full autonomy
(auto-grant ACP permission requests, edit + git allowed) · `-a` honored on all agent directives
(`::fix`, `::elaborate`, `::prompt`; **not** gated `::delete`).

## Why ACP over headless `claude -p`

- **Structured turn** — the client receives streaming `session/update` events (agent text, tool
  calls, plan) instead of scraping stdout; turn end is an explicit `stopReason`, not an exit code.
- **Permission as a message** — full autonomy is auto-answering `session/request_permission` with
  the allow option. Auditable and revocable per call, versus an opaque `--dangerously-skip-permissions`.
- **Editor-agnostic and future-proof** — the same protocol Zed/Casablanca can drive; swapping
  Claude Code for another ACP agent (codex-acp, etc.) is a config change, not a rewrite.
- **Progress surface** — the streamed events feed a future menulet/Casablanca "working…" view.

## Lifecycle (the state machine)

```
::fix -a banana                      user types, saves
      │  daemon debounce (~300ms) → rescan
      ▼
::fix[happy-otter:working] banana    CLAIM: mint id, set status=working, strip -a
      │  (own-write suppressed; in-flight map keyed by id)
      │  compile single marker → prompt + "remove this line & commit"
      ▼
ACP: spawn adapter → initialize → session/new (cwd = repo root) → session/prompt
      │  stream session/update (agent msgs, tool calls)
      │  session/request_permission → auto-grant (full autonomy)
      ▼
turn ends with stopReason
      ├─ end_turn → line already gone = resolved (defensive: if line remains, set :done)
      └─ error / refusal / cancelled → rewrite status to :failed (visible, not re-dispatched)
```

Guards against re-dispatch: (a) the `working`/`done`/`failed` status means "not a fresh
candidate"; (b) an in-memory in-flight set keyed by marker id; (c) `OwnWriteTracker` already
suppresses rescans from Indiana's own writes.

## Changes

### Core engine (`crates/core/`) — the grammar and write path

- **`parser.rs`** — add flag parsing to `parse_candidate` (~line 197): after the optional
  `[id:status]` bracket and before the free-text message, consume leading `-a` / `--auto`
  tokens and set a new `Marker.auto: bool`; the remainder is the message. Unknown `-x` tokens
  stop flag scanning and fall into the message (backward compatible). Add `Working` to the
  `Status` enum (line 13) and to `Status::parse` (line 20).
- **`id.rs`** — `is_valid_status` (line 101) accepts `"working"` alongside `done`/`failed`.
- **`markers.rs`** — add `is_auto_runnable(kind)` → true for `Fix`/`Elaborate`/`Prompt` (or an
  `auto_runnable: bool` column on `MarkerSpec` to keep it table-driven per IN_PRINCIPLES
  "one table drives everything").
- **`write.rs`** — new claim write beside `inject`/`normalize_line`: given (path, line, status),
  mint the id if absent, set the bracket to `[id:working]`, and strip a trailing `-a`/`--auto`
  flag token — atomic, mtime-guarded, idempotent (a claimed line re-runs byte-identical). Reuse
  `id.rs::format_bracket` / `parse_bracket`. Still the single write chokepoint.
- **`index.rs` / `compile.rs`** — thread `auto` through `Located` and `CompiledMarker`; expose a
  way to compile **one** marker's prompt (reuse `compile_with_options` filtered to the marker, or
  a small `compile_one`) so the ACP prompt is the same text a paste would carry.

### Daemon dispatcher — the ACP client (`crates/indiana/`)

- **New `crates/indiana/src/acp.rs`** — the ACP client, native Rust:
  - Depend on Zed's official **`agent-client-protocol`** Rust crate (client role). Pin/verify the
    version at implementation; mirror the message flow the vendored `CodexACPProtocol.ts` uses.
  - Resolve the **agent adapter** binary the way `casablanca/src/main/lib/indiana.ts` resolves
    `indiana` (explicit standard locations + PATH), overridable via config. Default adapter:
    Claude Code's ACP adapter (`claude-code-acp`, run via node/npx). This node adapter is the one
    external runtime dep; Indiana's own binary stays static.
  - Per dispatch: spawn the adapter, `initialize`, `session/new { cwd: <owning root> }`,
    `session/prompt { <compiled prompt> }`. Drive the connection to completion, forwarding
    `session/update` events to `~/.indiana/dispatch/<id>.log`.
  - **Permission handler = full autonomy:** answer `session/request_permission` by selecting the
    allow / allow-always option so edits and `git` run without a human. (Later: a per-repo policy.)
  - Detect turn end via `stopReason`. Tear down the session/subprocess after the turn.
- **New `crates/indiana/src/dispatch.rs`** (or fold into `acp.rs`) — orchestration/state:
  track in-flight turns in `HashMap<marker_id, DispatchHandle{ root, path, line, child }>`; cap
  concurrent dispatches (`MAX_INFLIGHT`, e.g. 3); one dispatch per id; on completion resolve
  (defensively set `:done` if the line remains) or on failure chokepoint-write `:failed`. Both
  writes recorded in `OwnWriteTracker`.
- **`daemon.rs`** — after each debounced rebuild (`spawn_watch_thread`, ~line 152), find
  candidates (`auto == true`, auto-runnable kind, no working/done/failed status, id not in-flight),
  claim each via the new write, then hand to `dispatch`. Startup policy: a `:working` marker with
  no live turn (daemon restarted mid-run) is re-dispatched best-effort.
- **`config.rs`** — extend `Config` (currently just `folders`) with an `agent` block describing the
  **ACP adapter** command + args + env (Claude auth), and an `auto_run: bool` kill-switch (the
  roadmap's "pausable"). Default adapter `claude-code-acp`; feature gated so it is opt-in until
  proven.

### Content / templates

- **`crates/core/templates/`** — the dispatched prompt appends an auto-run coda instructing the
  agent to delete the triggering marker line and commit. Put it in a small embedded
  `autorun_coda.md` (or extend `preamble.md`) so wording stays data, not code
  (IN_PRINCIPLES "content is data").

### Specs (spec wins or spec changes — never silent drift)

- **`docs/ARCHITECTURE.md:54`** — amend the invariant: the pure core never runs an agent; the
  **daemon** may, only when a marker opts in with `-a`, **over ACP**. Move handoff step 3 to
  "in progress."
- **`docs/indiana/IN_COMMANDS.md`** — document the `-a` / `--auto` flag on agent directives.
- **`docs/indiana/IN_LINE.md`** — add `working` as a third status; note it rides agent directives
  under auto-run, not only `::action`/`::todo`.
- **`docs/indiana/IN_IDENTITY.md`** — new rule: auto-run **mints identity at dispatch** (a plain
  `::fix` stays ephemeral; `::fix -a` becomes tracked the moment it is claimed).
- **New `docs/indiana/IN_AUTORUN.md`** — the lifecycle, the ACP client role and message flow,
  adapter resolution, the permission policy (full autonomy = auto-grant), concurrency cap, pause
  switch, failure and restart policy.
- **`docs/indiana/IN_TEST.md`** + **`ACTION_PLAN.md`** — new E-criteria; mark Phase 8 started.

## Verification

- **Unit (core):** `-a`/`--auto` parses to `auto=true` and is stripped from the message; a
  legitimate leading dash in a message is untouched; `working` parses, validates, and repairs;
  the claim write turns `::fix -a banana` into `::fix[<id>:working] banana` and is idempotent on
  re-run (byte-identical). Confirm existing byte-stability tests still pass.
- **Integration (daemon, mock ACP agent):** ship a **mock ACP agent** script (a small node/stdio
  program modeled on nimbalyst's `mockCodexAcpAgent.mjs`) that speaks ACP, requests one edit
  permission, removes the marker line, runs `git commit`, and ends the turn. Point `config.agent`
  at it. Fixture repo with `::fix -a`: assert the line becomes `[id:working]`, `session/new` gets
  cwd = repo root, the prompt carries the compiled text, the permission request is auto-granted,
  and on `end_turn` the marker is resolved; assert a turn that errors yields `[id:failed]`; assert
  the concurrency cap and one-dispatch-per-id hold. Deterministic — no network, no real Claude.
- **End-to-end (manual, dogfood):** set `config.agent` to the real `claude-code-acp` adapter, type
  `::fix -a fix this typo` in a scratch file in this repo, save, and watch the daemon claim,
  open an ACP session, and the fix + commit land. Run via the `run`/`verify` skill; inspect
  `~/.indiana/dispatch/<id>.log` for the streamed `session/update` trail.
- Keep the E11 watch tests' flakiness caveat in mind (`IN_TEST.md:154`); gate the ACP integration
  test behind the mock agent so it stays deterministic.

## Out of scope / follow-ups

- `::delete -a` (auto + gated confirmation conflict) — excluded by decision.
- Commit-message quality and the vendored nimbalyst commit machinery — being fixed separately.
- A menulet/Casablanca "working…" indicator reading the new `:working` status and the streamed
  ACP events (natural next step).
- Per-repo permission policy (prompt vs auto-grant) instead of global full autonomy.
- Additional ACP agents beyond Claude Code (codex-acp, etc.) — config `agent` block already
  leaves room.
