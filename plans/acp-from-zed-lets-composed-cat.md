# Make auto-run actually work: per-repo opt-in + Casablanca wiring + real ACP

## Context

Auto-run (`::fix -a` → daemon dispatches to Claude Code over ACP) is built and
unit/mock-tested, but three things stop it from working in practice:

1. **It's only been run against a mock ACP agent** — the real `claude-code-acp`
   path is unproven, so a live run will surface protocol/auth mismatches to fix.
2. **Casablanca never tells the daemon to monitor a folder.** Selecting a folder
   updates only Casablanca's own `userData` config; `indiana add` (which
   registers the folder in `~/.indiana/config.json`, scaffolds `.indiana/`, and
   live-monitors) is never called. So the daemon never watches Casablanca repos.
3. **Auto-run is a single global switch.** We want it **per-repo and
   committable**, so opening a repo in Casablanca turns auto-run on for *that*
   repo and the opt-in travels with the repo.

Goal (dev mode — ignore the released build): open a folder in Casablanca → it's
monitored, scaffolded, and auto-run is on → typing `::fix -a` fixes and commits
via real Claude Code, hands-free.

Decisions locked: per-repo committable auto-run opt-in; Casablanca folder-open
monitors + enables auto-run automatically.

> Note: the tree is concurrently gaining a `group`/`RunGroup` feature (new
> fields on `Located`/`CompiledMarker`, a protocol variant). Rebase these
> changes onto that; the dispatch touch-points below are the same regardless.

## Part A — Auto-run opt-in goes per-repo (daemon)

Today `Dispatcher::consider` (`crates/indiana/src/dispatch.rs`) gates on the
global `config.auto_run`. Change the gate to **per-repo, with the global flag as
fallback default**:

- Effective rule: a marker dispatches when `per_repo_autoRun ?? config.auto_run`
  is true, where `per_repo_autoRun` is the owning repo's `.indiana/casablanca/
  settings.json` `autoRun` key. Per-repo `true`/`false` overrides the global
  default; unset falls back to global (still default off). This keeps the global
  switch as a master default and makes per-repo the primary control.
- In `consider`, compute each candidate's owning root (the existing
  `owning_root` helper) and read `autoRun` via the settings module
  (`crates/indiana/src/casablanca.rs::get(root, "autoRun")` → expect a JSON
  bool). Reuse what's there; no new file format.
- The ACP adapter config (`config.agent` command/args/env) **stays global** in
  `~/.indiana/config.json` — it's a machine-level tool, not per-repo.
- `IN_AUTORUN.md` + `IN_FOLDER.md`: document `autoRun` as the per-repo opt-in
  living in `.indiana/casablanca/settings.json`, read by the daemon.

## Part B — Casablanca folder-open monitors + sets up (Electron)

Reuse the existing `resolveIndianaBinary()` + `execFile` pattern in
`crates/casablanca/src/main/lib/indiana.ts` (same as `copyAllMarkers`). Add:

- `ensureMonitored(rootPath)` — shells `indiana add <rootPath>` (idempotent:
  registers in `~/.indiana/config.json`, scaffolds `.indiana/`, live-adds to a
  running daemon). Best-effort with a friendly toast if `indiana` is missing,
  mirroring `copyAllMarkers`.
- Enable auto-run on **first add**: write `autoRun: true` into the repo's
  settings via the existing `writeRepoSetting(rootPath, 'autoRun', true)`
  (`main/lib/repo-settings.ts`) — no subprocess needed. Only on `PROJECTS_ADD`,
  not on every re-select, so a later manual disable is respected.

Wiring in `crates/casablanca/src/main/ipc.ts`:
- `PROJECTS_ADD` handler: after `addProject`, call `ensureMonitored` +
  enable auto-run.
- `adopt()` (runs on add, switch, and initial active load): call
  `ensureMonitored` so any active repo is watched when Casablanca launches or
  switches. It sits naturally beside the existing `ensureRepo(vault)` (git init).

Result: point 2 ("repo added to `~/.indiana/config.json` when monitored") falls
out of `indiana add`; point 3 ("select a folder → monitor + setup") is the
`adopt`/`PROJECTS_ADD` wiring.

## Part C — Get it working end-to-end against real Claude Code (dev)

Runbook, then iterate against the real adapter (only the mock has been exercised):

1. **Build + run from source**: `cargo build --release`; run the daemon in a
   terminal (`indiana serve <repo>` or via config) so the spawned
   `claude-code-acp` inherits your shell env — Node on PATH and Claude Code auth.
   (A launchd/menulet daemon may lack that env; dev = run it in a terminal.)
2. **Adapter reachable + authed**: confirm `npx -y @zed-industries/claude-code-acp`
   starts and Claude Code is logged in (subscription) or `ANTHROPIC_API_KEY` is
   set; if needed, put creds in `config.agent.env`.
3. **Opt in**: `.indiana/casablanca/settings.json` `{"autoRun": true}` (or open
   the repo in Casablanca once Part B lands).
4. **Drive it**: type `::fix -a fix this typo`, save, watch
   `~/.indiana/dispatch/<id>.log` for the ACP conversation.
5. **Fix real-adapter mismatches** found in the log — likely spots in
   `crates/indiana/src/acp.rs`: the `initialize` params/`clientCapabilities`
   shape, an `authenticate` step if the adapter reports `authMethods`, the
   `session/prompt` content-block format, the permission-option `kind` strings
   the real adapter offers, `stopReason` values, and whether the agent commits
   itself vs needs the coda tightened. Adjust and re-run until the marker line
   is removed and a commit lands.

## Files

- `crates/indiana/src/dispatch.rs` — per-repo gate in `consider` (reuse
  `casablanca::get`, `owning_root`).
- `crates/indiana/src/acp.rs` — real-adapter fixes as surfaced by step C5.
- `crates/casablanca/src/main/lib/indiana.ts` — `ensureMonitored`.
- `crates/casablanca/src/main/ipc.ts` — call `ensureMonitored` in `adopt`;
  enable auto-run on `PROJECTS_ADD`.
- `crates/casablanca/src/main/lib/repo-settings.ts` — reuse `writeRepoSetting`
  (already present).
- Docs: `IN_AUTORUN.md`, `IN_FOLDER.md` (per-repo `autoRun`).

## Verification

- **Rust unit**: `consider` dispatches only when the owning repo's `autoRun` is
  true (extend the dispatch/mock-agent integration test — add a repo with
  `autoRun:true` and one without; only the former resolves). Global-fallback
  case covered too. Run `cargo test --workspace` and the feature-gated
  `--features test-support --test autorun`.
- **Casablanca**: `npm run typecheck` + `vitest run` in `crates/casablanca`
  (needs Node — the TS from the prior session is still unverified there too).
  Manually: open a fresh folder in the dev editor, confirm it appears in
  `~/.indiana/config.json`, `.indiana/` is scaffolded, and
  `.indiana/casablanca/settings.json` has `autoRun:true`.
- **End-to-end (the real goal)**: with a terminal daemon + Claude auth, open a
  scratch repo in Casablanca, type `::fix -a`, and watch the fix + commit land;
  confirm via the dispatch log and `git log`. This is the acceptance test.

## Out of scope / follow-ups
- launchd/menulet daemon env + auth for auto-run (dev uses a terminal daemon).
- A visible auto-run on/off toggle in the editor UI (chose auto-enable for now).
- Moving `autoRun` to a face-neutral `.indiana/settings.json` if the coupling of
  the daemon to a `casablanca/`-named file grates later.
