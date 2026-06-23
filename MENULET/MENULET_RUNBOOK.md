---
status: draft
purpose: Break M12 menulet work into small verifiable implementation steps.
approval: pending
---

# MENULET_RUNBOOK

## Assumptions
- Tauri 2 is the app shell (stable as of 2026; tray API, sidecar, and `LSUIElement` all mature).
- Indiana remains the only scanner, compiler, counter, and config owner. The menulet renders; it never groups, counts, or compiles — including per-folder counts (see [Caveats](#caveats)).
- Menulet may write `~/.indiana/focus.txt`; that is human scratch, not Indiana state.
- The menulet duplicates only the daemon's primitive-typed protocol structs (`Request`, plus `StatusResponse`, `AddResponse`, `RemoveResponse`, `CopyResponse` defined here). It never sees `Index` or `CompiledPayload` — the daemon returns counts and rendered text instead. This keeps `indiana_core` out of the Tauri build (no second core build) and makes duplication trivial. Where a daemon response carries extra fields (e.g. `AddResponse.index`, for CLI ergonomics), the menulet's mirror omits them; serde ignores unknown fields on deserialize.
- The tray icon is a monochrome template PNG (not PDF — Tauri's tray takes a raster `Image`; macOS auto-tints it via `icon_as_template(true)`). Exact glyph deferred to visual polish; a placeholder must exist for the scaffold to render.
- The menulet only stops the daemon it spawned itself. Any daemon already alive at launch (a manual `indiana serve` or a launchd-managed one) is treated as externally managed: the panel shows status but offers no stop. This is simpler than sniffing the daemon's parentage over the socket, and correct — `launchd` runs with `KeepAlive=true` (service.rs), so a Shutdown would just be restarted.

---

## M12.0 — Protocol gap fill (daemon-side prerequisite)

The current protocol serves `Scan`, `Payload`, `Add`. The menulet needs `Status`, `Remove`, `Copy`, and `Shutdown`. All new responses carry primitives only, so the menulet never imports core.

### M12.0.1 — Status (folders + per-folder counts)
- Add `Request::Status` to `crates/indiana/src/protocol.rs`.
- Add `FolderInfo { path: String, count: usize }` and `StatusResponse { folders: Vec<FolderInfo> }`.
  - The daemon owns the grouping: for each monitored root, count the held index's markers whose path is under that root. This is the count the menulet displays without ever touching `Index`.
  - No `running` / `socket_path` field: a successful Status response is itself proof the daemon is alive; the menulet already knows the socket path.
- Wire into `daemon.rs` `handle()`: read roots + index snapshot under the existing locks, build the `Vec<FolderInfo>`.
- Add `client_status() -> Option<StatusResponse>` (None if socket dead).
- Verify: `cargo test` — `StatusResponse` serde round-trip; live daemon with two added folders reports both with correct counts.

### M12.0.2 — Shutdown
- Add `Request::Shutdown` to `protocol.rs`.
- In `daemon.rs` `handle()`: on `Shutdown`, write the ack, flush, remove the socket file (`std::fs::remove_file(socket_path())`), then `std::process::exit(0)`. Unlinking first means the next `serve` binds cleanly without leaning on stale-socket recovery.
- Add `client_shutdown() -> bool` (true if daemon acknowledged before exiting).
- Verify: spawn daemon, send `Shutdown`, assert the process is gone and a fresh `connect` is refused within 2s.

### M12.0.3 — Remove folder
- Add `Request::Remove { path: PathBuf }` to `protocol.rs`.
- Add `RemoveResponse { removed: bool, index: Index }` (mirrors `AddResponse` for CLI symmetry; the menulet's mirror keeps only `removed`).
- Implement `remove_folder_live()` in `daemon.rs`: remove from `Config::folders`, persist config, unwatch the root (`deb.watcher().unwatch(root)` + `deb.cache().remove_root(root)` — the inverse of `watch_root`; there is one shared debouncer, not one per root), rebuild the index from the remaining roots.
- Add `client_remove(path) -> Option<RemoveResponse>` and `Config::remove_folder(&Path) -> bool`.
- Add `indiana remove <path>` CLI subcommand (same shape as `indiana add`, including the daemon-down fallback that mutates config for the next `serve`).
- Verify: `indiana add` a folder, `indiana remove` it, assert it is gone from config and absent from `indiana scan`.

### M12.0.4 — Copy (per-folder rendered bundle)
- Add `Request::Copy { path: PathBuf }` to `protocol.rs`.
- Add `CopyResponse { text: String }`.
  - The daemon filters the held index to that folder's markers, `compile`s, and `render_text`s — the same pipeline as `indiana copy` (main.rs), scoped to one root. The menulet receives ready-to-paste text and never sees `CompiledPayload`.
- Add `client_copy(path) -> Option<String>`.
- Verify: add a folder with known markers, call Copy, assert the returned text matches `indiana copy <folder>`.

---

## M12.1 — App Scaffold

### M12.1.1 — Create Tauri 2 project
- Create `menulet/` at repo root, outside the Cargo workspace.
- Scaffold: `npm create tauri-app@latest menulet` with vanilla HTML/JS frontend, or manual Tauri 2 init.
- Key `tauri.conf.json` settings:
  - `productName`: `"Indiana"`
  - `identifier`: `"com.indiana.menulet"`
  - `app.withGlobalTauri: true`
  - macOS `"LSUIElement": true` (accessory mode, no Dock icon, no app menu)
  - Sidecar: `"externalBin": ["binaries/indiana"]`
- Add the Tauri 2 plugins the panel needs: `tauri-plugin-dialog` (directory picker), `tauri-plugin-shell` (spawn sidecar), and a clipboard path — either `tauri-plugin-clipboard-manager` or `arboard` in the backend (pick one; clipboard plugin avoids the extra crate).
- Add a placeholder monochrome template PNG tray icon at `menulet/icons/tray.png` (≈18×18 pt @2x, alpha mask).

### M12.1.2 — Build pipeline
- Script or Makefile target `build-menulet`:
  1. `cargo build --release --target aarch64-apple-darwin` → `target/aarch64-apple-darwin/release/indiana`
  2. `cp target/aarch64-apple-darwin/release/indiana menulet/src-tauri/binaries/indiana`
  3. `cd menulet && npm install && npx tauri build`
- Verify: `npm install` succeeds; `npx tauri build` produces `Indiana.app`; `ls Indiana.app/Contents/MacOS/indiana` shows the sidecar binary.

### M12.1.3 — Tray + window setup
- In `src-tauri/src/main.rs`: configure a tray (`TrayIconBuilder` with the template PNG + `icon_as_template(true)`) and a hidden window.
- Window: `decorations: false`, `always_on_top: true`, `visible: false`, `skip_taskbar: true`.
- Tray click → toggle window visibility; position under the menu-bar icon from the `TrayIconEvent` rect.
- Window blur → hide.
- Verify: `cargo tauri dev` shows the icon in the menu bar; click toggles an empty panel under it; clicking away hides it.

---

## M12.2 — Socket Client (Tauri backend)

### M12.2.1 — Duplicate protocol types (primitives only)
- In `menulet/src-tauri/src/protocol.rs`: duplicate `Request` and the menulet-facing responses `StatusResponse` (+ `FolderInfo`), `AddResponse { added: bool }`, `RemoveResponse { removed: bool }`, `CopyResponse { text: String }`.
- None reference `Index` or `CompiledPayload`. The daemon's `AddResponse`/`RemoveResponse` carry an extra `index` field; serde drops it when deserializing into these slimmer mirrors (do not set `deny_unknown_fields`).
- Derive `#[derive(Debug, Clone, Serialize, Deserialize)]` matching the originals' `#[serde(tag = "cmd", rename_all = "lowercase")]` on `Request`.

### M12.2.2 — Socket client functions
- In `menulet/src-tauri/src/socket.rs`, mirror the daemon's client functions:
  - `status() -> Option<StatusResponse>` — send `Status`; doubles as the liveness probe (None ⇒ daemon down).
  - `add_folder(path: &Path) -> bool` — send `Add`, read `added`.
  - `remove_folder(path: &Path) -> bool` — send `Remove`, read `removed`.
  - `copy_folder(path: &Path) -> Option<String>` — send `Copy`, read `text`.
  - `shutdown() -> bool` — send `Shutdown`, read ack.
- All share a helper `send_recv(req: &Request) -> Option<String>` (connect, write line, read line).
- Socket path: `~/.indiana/indiana.sock`, respecting `INDIANA_HOME` (same rule as paths.rs).
- Verify: backend tests against a real `indiana serve` child — add a folder, assert `status()` shows it with the right count, `copy_folder` returns non-empty text, `remove_folder` then drops it.

### M12.2.3 — Tauri commands
- Register `#[tauri::command]` wrappers in `main.rs`, each <10 lines of glue:
  - `status` → `StatusResponse` (folders + counts; absence ⇒ stopped).
  - `add_folder(path: String) -> bool`
  - `remove_folder(path: String) -> bool`
  - `copy_folder(path: String)` → fetch text via `copy_folder`, write to clipboard.
  - `shutdown() -> bool`
  - `spawn_sidecar()` → spawn the bundled `indiana serve` via the shell plugin.
  - `read_focus() -> String`
  - `save_focus(text: String)` → persist to `~/.indiana/focus.txt`.
- No parsing, counting, or compiling in any command.
- Verify: invoke each from the Tauri dev console against a live daemon; assertions pass.

---

## M12.3 — Panel UI

Implement the layout from [MENULET_UI.md](MENULET_UI.md).

### M12.3.0 — Initial UI (first deliverable)
The first screen we expect to see, after launch with no folders yet monitored, is exactly two lines:
- `● Server running` — the daemon is up (the sidecar spawned, or one was already alive).
- `Select folder to monitor →` — the empty-state action; click opens the directory picker (`add_folder`).

This is the "Running + empty" state and nothing more — no folder list, counts, or focus polish needed to call M12.3 done. Everything below (full folder list, copy, focus persistence) layers onto this screen. If launch can show those two lines and the picker adds a folder that then appears in the list, the menulet works end to end.

### M12.3.1 — HTML structure
- Single `index.html` with three sections:
  1. Focus field: `<input type="text" id="focus" placeholder="What are you working on?">`
  2. Folder list: `<ul id="folders">` with an `<li class="folder">` per monitored folder. Each row: `<span class="basename">`, `<span class="count">`, click-to-copy, right-click-to-remove.
  3. Server control: `<div id="status">` with status dot (● green / ○ grey), label, start/stop button.
- `[+]` button beside the "MONITORED FOLDERS" header.
- Empty state: when `#folders` is empty, show a centered "Monitor a folder →" prompt.

### M12.3.2 — CSS
- Fixed-width panel (~320px), system font, light background with subtle border.
- Compact rows; no scrolling unless >10 folders.
- Status bar at bottom, separated by a hairline.
- Focus input full-width with bottom border.

### M12.3.3 — JavaScript state machine
- On load: `invoke('status')`.
  - Some ⇒ render folder list + counts + green dot (running). Empty `folders` ⇒ empty-state prompt + green dot.
  - None ⇒ grey dot + start button (stopped).
- States:
  - Running + folders: full layout, counts visible.
  - Running + empty: focus field + empty-state prompt + green dot.
  - Stopped: folder list greyed out (counts hidden), grey dot + start button.
  - Connecting: spinner + "Starting…" (transient, while sidecar spawns).
- Events:
  - Focus field: save on blur + Enter via `invoke('save_focus', { text })`.
  - `[+]` click: native directory picker via `tauri-plugin-dialog`, then `invoke('add_folder', { path })`, then re-`invoke('status')`.
  - Folder row click: `invoke('copy_folder', { path })` → brief "Copied ✓" flash.
  - Folder row right-click: context menu → "Stop monitoring" → `invoke('remove_folder', { path })`, then re-`invoke('status')`.
  - Stop button: `invoke('shutdown')` → stopped state (only shown for a daemon we spawned — see M12.5.2).
  - Start button: `invoke('spawn_sidecar')` → poll `status` every 500ms until Some.
- Verify: component test — render each state, click through transitions, assert correct invocations (mock `invoke`, or test against a running daemon).

### M12.3.4 — Focus text persistence
- `read_focus` on load; `save_focus` on blur / Enter / close.
- Backend `save_focus`: write `~/.indiana/focus.txt` atomically (tempfile → rename). Respect `INDIANA_HOME`.
- Backend `read_focus`: file contents or empty string.
- Verify: integration test — write focus text, close panel, reopen, assert restored.

---

## M12.5 — Sidecar Lifecycle

### M12.5.1 — Connect-or-spawn on launch
- On the Tauri `setup` hook:
  1. `status()`.
  2. Some ⇒ running; render folders. Mark the daemon as not-ours (we did not spawn it).
  3. None ⇒ spawn the bundled sidecar via the shell plugin (`app.shell().sidecar("indiana")?.args(["serve"]).spawn()`). Mark the daemon as ours.
  4. Poll `status` every 500ms up to 10s. Some ⇒ running. Timeout ⇒ "Failed to start" error state.
- PATH detection (DISTRO.md): before spawning the bundled binary, check `~/.local/bin/indiana`, `/usr/local/bin/indiana`, `$(brew --prefix)/bin/indiana`. If any exists and is newer (mtime), spawn it instead, so power users can upgrade the daemon independently.

### M12.5.2 — Process management
- Track the spawned child (`CommandChild`) and whether the live daemon is ours.
- Stop button:
  - Daemon is ours ⇒ show stop. `shutdown()` over the socket; wait up to 2s for the socket to close; if still alive, `child.kill()`.
  - Daemon is not ours (alive at launch, manual or launchd) ⇒ hide stop; show "Managed externally". We never kill a daemon we did not start; launchd would restart it regardless (`KeepAlive=true`).
- Verify: spawn via sidecar, click stop, assert the process exits. Start daemon manually before launching the menulet, assert stop is hidden.

### M12.5.3 — Crash recovery
- Poll `status` every 30s while "running".
- None (connection refused / timeout):
  - Daemon was ours ⇒ "Server stopped" state with a "Restart" option.
  - Daemon was not ours ⇒ "Server unreachable" message (do not auto-respawn what we did not own).
- Restart re-runs the M12.5.1 connect-or-spawn logic.
- Verify: kill the daemon, assert the panel transitions within 30s. Click restart, assert respawn.

---

## M12.6 — End-to-end acceptance (does it actually work?)

Two layers: a scripted protocol smoke test that needs no GUI (CI-able), and a short manual GUI pass that only a human can see. The menulet is "working" when both pass.

### M12.6.1 — Scripted smoke test (no GUI)
The whole daemon contract the menulet depends on is reachable over the socket, so it can be exercised headless. Save as `menulet/scripts/smoke.sh`; needs `nc -U` and `jq` (both on macOS).

```bash
#!/usr/bin/env bash
# End-to-end check of the daemon contract the menulet relies on. No GUI.
set -euo pipefail

BIN="target/aarch64-apple-darwin/release/indiana"
export INDIANA_HOME="$(mktemp -d)"          # isolate from the real daemon
WATCH="$(mktemp -d)"; printf '::fix do it\n::action ship\n' > "$WATCH/notes.md"
SOCK="$INDIANA_HOME/indiana.sock"
send() { printf '%s\n' "$1" | nc -U "$SOCK"; }   # one JSON line in, one out

"$BIN" serve & DAEMON=$!
trap 'kill $DAEMON 2>/dev/null || true; rm -rf "$INDIANA_HOME" "$WATCH"' EXIT
for _ in $(seq 1 30); do [ -S "$SOCK" ] && break; sleep 0.1; done

"$BIN" add "$WATCH" >/dev/null                                    # add a folder
[ "$(send '{"cmd":"status"}' | jq '.folders | length')" = 1 ]    # status lists it
[ "$(send '{"cmd":"status"}' | jq '.folders[0].count')" = 2 ]    # daemon-side count
send "{\"cmd\":\"copy\",\"path\":\"$WATCH\"}" | jq -e '.text | length > 0' >/dev/null  # copy text
send "{\"cmd\":\"remove\",\"path\":\"$WATCH\"}" >/dev/null        # remove it
[ "$(send '{"cmd":"status"}' | jq '.folders | length')" = 0 ]    # gone
send '{"cmd":"shutdown"}' >/dev/null || true                     # daemon exits, unlinks socket
sleep 1; [ ! -S "$SOCK" ] && echo "SMOKE OK"
```

- Verify: `bash menulet/scripts/smoke.sh` prints `SMOKE OK` and exits 0.
- Wire it into the Makefile (`make smoke`) and CI after `cargo build`. This proves the M12.0 protocol additions before any GUI work, and is the regression guard if the daemon changes.

### M12.6.2 — Manual GUI acceptance
The smoke test covers the wire; only a person can confirm the panel. With the daemon stopped, launch `Indiana.app` and walk:

1. Cold launch → menu-bar icon appears; click it → panel shows the [Initial UI](#m1230--initial-ui-first-deliverable): `● Server running` + `Select folder to monitor →`. (Proves auto-spawn + Status.)
2. Click the prompt → native picker → choose a folder with `::` markers → it appears in the list with the right count. (Proves add + status refresh.)
3. Click the folder row → "Copied ✓" flash → paste elsewhere → bundle text is on the clipboard. (Proves copy.)
4. Right-click → "Stop monitoring" → row disappears. (Proves remove.)
5. Type in the focus field, close the panel, reopen → text restored. (Proves focus persistence.)
6. Click stop (visible only because the menulet spawned this daemon) → dot goes grey, start button shows. Click start → back to running. (Proves lifecycle + own-child rule.)
7. Click away from the panel → it hides. (Proves blur-to-hide.)

- Verify: all seven steps pass on a clean machine (no pre-existing daemon). Record pass/fail per step; any fail blocks the milestone.

### M12.6.3 — Definition of done
- `make smoke` green (M12.6.1).
- All seven manual steps pass (M12.6.2).
- Every [Verification gate](#verification-gates) below is met.

---

## Deferred
- Code signing and notarization.
- Auto-update (Sparkle / Tauri updater).
- Per-kind breakdown in folder rows (single total for v1).
- Visual polish beyond the v1 panel.
- Responsive panel width / dark mode.

---

## Caveats

- Per-folder counts placement. `Index` is one flat marker list across all roots (index.rs, daemon.rs `build_index`). Grouping must live in the daemon (M12.0.1), not the menulet, to honour "the menulet never counts" (MENULET_PRD, MENULET_UI). This is the load-bearing reason the protocol grew a `Status` and a `Copy` response instead of shipping the raw `Index`/`CompiledPayload`.
- Tauri 2 API drift. Earlier drafts named Tauri-1 paths. In Tauri 2: dialog/clipboard/shell are plugins (`tauri-plugin-*`), the sidecar is `app.shell().sidecar(..)` (not `Command::new_sidecar`), and tray icons are raster `Image`s with `icon_as_template(true)` (not PDFs). Verify exact names against the installed Tauri version when scaffolding.
- NSPanel fidelity. A plain Tauri window (`always_on_top` + `skip_taskbar` + `decorations:false`) approximates a menu-bar panel but is not a true non-activating `NSPanel`. If blur/focus behaviour misbehaves, adopt the `tauri-nspanel` plugin or the native pattern from `old/adhd-menulet-focus-finder` (referenced in MENULET_PRD).
- Copy phasing. MENULET_PRD "Phase 1" still says copy lands later, but GOAL/PHASES Phase 5 and this runbook treat one-click copy as core to the menulet (M12.0.4, M12.2.3). Reconcile MENULET_PRD if copy is truly first-deliverable.

---

## Build order (dependency graph)

```
M12.0.1 Status ──┐
M12.0.2 Shutdown ─┤
M12.0.3 Remove ───┤── M12.2.2 client ── M12.2.3 commands ──┐
M12.0.4 Copy ─────┘                                          │
                                                              ├── M12.3.3 wiring + M12.5 lifecycle
M12.1.1 scaffold ── M12.1.2 build ── M12.1.3 tray/window ──┤
                                                              │
M12.3.1 HTML ── M12.3.2 CSS ── M12.3.3 state machine ──────┘
                                    │
M12.3.4 focus persistence ──────────┘
```

- M12.0.x (4 tasks) — parallel, pure Rust in `crates/indiana`.
- M12.1.x (3 tasks) — sequential within; can start in parallel with M12.0.
- M12.2.x (3 tasks) — sequential within; needs M12.0 done.
- M12.3.x (4 tasks) — HTML/CSS parallel, then JS state machine. Needs M12.1.3 for window layout; M12.2.3 for IPC.
- M12.5.x (3 tasks) — sequential within; needs M12.2.3 + M12.3.3.
- M12.6.1 smoke test — write it right after M12.0 (it only needs the daemon); it then guards every later change. M12.6.2 manual pass runs last, once the panel exists.

## Verification gates

| Gate | What proves it |
|------|---------------|
| Protocol complete | `indiana remove` works end-to-end; `indiana serve` answers `Status` (folders + counts), `Copy` (rendered text), and exits + unlinks socket on `Shutdown` |
| App builds | `npx tauri build` produces unsigned `Indiana.app` with sidecar at `Contents/MacOS/indiana` |
| Socket client works | Backend tests pass against a live daemon: add, status, copy, remove all round-trip |
| Panel renders | Open panel — focus field, folder list (daemon-supplied counts), status bar render per MENULET_UI.md |
| Focus persists | Type text, close panel, reopen — text restored |
| Copy works | Click a folder row → clipboard holds the daemon-rendered bundle for that folder |
| Lifecycle | Launch auto-connects to a running daemon; no daemon ⇒ spawns sidecar; stop kills only the daemon we spawned; externally-managed daemon hides stop |
