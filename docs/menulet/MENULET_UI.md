---
status: draft
purpose: Specify the first menulet panel layout and states.
approval: pending
---

# Menulet — UI Plan (v2)

> Iteration on [MENULET_PRD.md](MENULET_PRD.md). Same constraints: NSPanel, hide-on-blur, Tauri 2, never computes.

---

## Layout (top → bottom)

```
┌─────────────────────────────────┐
│  + add folder              ⚙    │  ← toolbar: add left, theme cog right
├─────────────────────────────────┤
│  ~/projects/indiana  3⨯  run ⋯ │  ← folder row: path, count, run, three-dot menu
│  ~/projects/site     7⨯  run ⋯ │  │  ⋯ → update / replace indiana commands
│  ~/work/notes        0⨯  run ⋯ │  │  ⋯ → copy actions / remove folder
├─────────────────────────────────┤
│  ● server running     v0.1.0    │  ← footer: status left, version right
│    stop                         │
└─────────────────────────────────┘
```

### 1. Toolbar

- Add folder `[+ add folder]`: opens a native folder picker. Sends path to daemon over the socket.
- Theme cog `[⚙]`: opens a light/dark/system dropdown on the right.

### 2. Monitored folders

- List: each folder shows its basename (or tilde-path) and the live marker count Indiana reports.
- Add folder: done via the toolbar `[+ add folder]` button.
- Click a folder row: copies its compiled bundle to clipboard. Visual feedback: "copied" flash.
- Three-dot menu `[⋯]` per row:
  - `update indiana commands`: delegates to `indiana templates refresh <path>` via the sidecar. Creates missing `.indiana/indianas/<command>/prompt.md` files; existing files are left untouched.
  - `replace indiana commands`: delegates to `indiana templates replace <path>` via the sidecar. Rewrites every `.indiana/indianas/<command>/prompt.md` with the embedded default — destructive, discards user edits to command templates. `context-model/` and `chief-of-staff/` are not touched.
  - `remove folder`: sends remove command to daemon; daemon drops it from config.
- Empty state: when no folders are monitored, show a centered "monitor a folder…" prompt that triggers the add picker.

Count per folder is tallied by Indiana (actions, fixes, questions, etc.) — the menulet just displays the number. Full kind breakdown deferred; a single total is enough for v1.

### 3. Footer

Status and version at the bottom of the panel.

- Left: server status with dot, label, and start/stop button.
  - Running: green dot ● + "server running" + stop button.
  - Stopped: grey dot ○ + "server stopped" + start button.
  - Starting: spinner + "starting…" (transient while sidecar spawns).
- Right: app version (e.g. `v0.1.0`), sourced from Tauri app metadata (`getVersion()`).
- Stop: sends shutdown command over the socket. Graceful — finishes in-flight compile, doesn't kill abusively.
- Start: spawns the bundled `indiana serve` sidecar. Checks socket first; if a launchd daemon is already alive, just connects.

The server status reflects the socket connection — not a separate health check. If the socket is alive and responding, it's running.
---

## Persistence

| What | Where | Who writes |
|------|-------|------------|
| Monitored folders | `~/.indiana/config.json` | Indiana daemon (menulet sends command, daemon mutates) |
| Focus text | `~/.indiana/focus.txt` | Menulet directly (trivial, no daemon involvement) |

`config.json` shape — minimal:
```json
{
  "folders": ["/Users/niklas/projects/indiana", "/Users/niklas/projects/site"]
}
```
Focus is a separate file to keep the config clean — it's user scratch, not Indiana state.

---

## States to handle

| State | What the panel shows |
|-------|---------------------|
| Server running, 1+ folders | Full layout: toolbar, folder list with counts, footer with green dot + version |
| Server running, no folders | Toolbar + empty state ("monitor a folder…") + footer with green dot + version |
| Server stopped | Toolbar, folder list hidden, footer with grey dot + start button |
| Launch (first ever) | Toolbar, no folders, server stopped. |

---

## What changed from v2

- Moved: theme cog to top toolbar, add folder to top toolbar (was monitoring header).
- Removed: "monitoring" header label.
- Added: three-dot menu per folder row with `update indiana commands` and `remove folder` actions (was right-click remove).
- Added: `refresh_templates` Tauri command that delegates to `indiana templates refresh <path>` sidecar.
- Added: `replace_templates` Tauri command + "replace indiana commands" menu item, delegating to `indiana templates replace <path>` (destructive reset of command templates).
- Added: app version display in footer (`v0.1.0`) sourced from Tauri metadata.
- Moved: server status to bottom-left footer position (was top of panel).
