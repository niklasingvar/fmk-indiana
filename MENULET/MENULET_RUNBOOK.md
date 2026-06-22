---
status: draft
purpose: Break M12 menulet work into small verifiable implementation steps.
approval: pending
---

# MENULET_RUNBOOK

## Assumptions
- Tauri 2 is the app shell.
- Indiana remains the only scanner, compiler, counter, and config owner.
- Menulet may write `~/.indiana/focus.txt`; that is human scratch, not Indiana state.
- No old reference project exists in this repository.

## M12.1 — App Scaffold
- Create `menulet/` as a Tauri 2 app, outside the Cargo workspace.
- Configure accessory mode: no Dock icon, menu-bar item only.
- Bundle sidecar path as `Contents/MacOS/indiana`.
- Verify: `npm install` and `npm run tauri build` reach the Tauri build step.

## M12.2 — Socket Client
- Add a tiny Rust client module in the Tauri backend.
- Commands: `server_status`, `scan`, `payload`, `add_folder`, `remove_folder`, `copy_folder`.
- No parsing, counting, or compiling in the menulet.
- Verify: backend tests use a fake socket server.

## M12.3 — Panel UI
- Implement the layout from [MENULET_UI.md](MENULET_UI.md).
- States: stopped, starting, running with folders, running empty.
- Folder row click calls `copy_folder`.
- Verify: component tests for state rendering.

## M12.4 — Focus Text
- Persist one line to `~/.indiana/focus.txt`.
- Save on blur and Enter.
- Verify: read/write test against a temp `INDIANA_HOME` equivalent.

## M12.5 — Sidecar Lifecycle
- On launch: connect to existing socket; if absent, spawn bundled sidecar.
- Stop button only stops the child process the menulet spawned.
- Never kill a launchd daemon.
- Verify: process lifecycle test with a fake sidecar.

## Deferred
- Code signing and notarization.
- Auto-update.
- Per-kind breakdown in folder rows.
- Visual polish beyond the v1 panel.
