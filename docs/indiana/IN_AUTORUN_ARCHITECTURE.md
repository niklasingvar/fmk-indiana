---
status: draft
purpose: Map auto-run ownership, state, configuration, and duplicate-dispatch defenses.
approval: pending
---

# IN_AUTORUN_ARCHITECTURE — where agent runs live

> Behavior contract: [IN_AUTORUN.md](IN_AUTORUN.md). Casablanca follow surface: [../casablanca/CASABLANCA_AGENT_JOBS.md](../casablanca/CASABLANCA_AGENT_JOBS.md).

## End-to-end path
1. Marker grammar and `-a` parsing live in `crates/core/src/markers.rs` and `crates/core/src/parser.rs`.
2. Marker claim and failure writes live in `crates/core/src/write.rs`, the single source-mutation chokepoint.
3. `crates/indiana/src/daemon.rs` watches repos, trailing-edge debounces filesystem bursts for 300 ms, rebuilds the index, and offers candidates to the dispatcher.
4. `crates/indiana/src/dispatch.rs` owns claims, one-turn-per-repo leases, jobs, transcripts, and completion checks.
5. `crates/indiana/src/acp.rs` owns the ACP child process and JSON-RPC session.
6. `crates/indiana-protocol/src/lib.rs` owns the socket wire types consumed by faces.
7. Casablanca projects jobs through `main/lib/indiana.ts` → IPC/preload → `renderer/src/app/TopBar.tsx` and `JobFollowPopover.tsx`.
8. Casablanca adopts daemon claim writes without clobbering drafts in `renderer/src/storage/useVault.ts` and `shared/marker-claim.ts`.

## State and settings
- Marker status in markdown is durable dispatch truth: open → `working` → removed or `failed`.
- Live jobs and transcripts are daemon memory. Raw ACP logs live at `~/.indiana/dispatch/<id>.log`.
- Repo settings live at `.indiana/casablanca/settings.json`: `autoRun` enables dispatch; optional `model` selects the ACP session model.
- Machine settings live at `~/.indiana/config.json`: global `auto_run` fallback and ACP adapter `agent.command`, `args`, and `env`.
- Model selection occurs after `session/new` through ACP `session/set_config_option`; absent `model` leaves the adapter default unchanged.

## Why one edit does not launch several agents
- The watcher waits for 300 ms of quiet and rebuilds once per save burst.
- A repo lease permits one live turn per working tree; distinct repos may run in parallel.
- Claimed marker ids are held in an in-flight set, and `working`/`failed` markers are not fresh candidates.
- Daemon-owned marker writes are suppressed from the watcher.
- Casablanca splices claim brackets into its live draft so a later autosave cannot recreate an unclaimed marker.
