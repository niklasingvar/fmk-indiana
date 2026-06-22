---
status: draft
purpose: Specify the first menulet panel layout and states.
approval: pending
---

# Menulet — UI Plan (v2)

> Iteration on [MENULET_PRD.md](MENULET/MENULET_PRD.md). Same constraints: NSPanel, hide-on-blur, Tauri 2, never computes.

---

## Layout (top → bottom)

```
┌─────────────────────────────────┐
│  FOCUS ON                       │  ← text input, always visible
│  [________________________]     │
├─────────────────────────────────┤
│  MONITORED FOLDERS        [+]   │  ← add button
│  ~/projects/indiana       3⨯    │  ← folder path, marker count
│  ~/projects/site          7⨯    │  │  click → copy bundle
│  ~/work/notes             0⨯    │  │  right-click → remove
├─────────────────────────────────┤
│  ● Server running          ⏹    │  ← status dot + stop button
│            or                    │
│  ○ Server stopped          ▶    │  ← status dot + start button
└─────────────────────────────────┘
```

### 1. FOCUS ON

A single-line text field at the top. The user types what they're working on right now — a reminder, a task name, a branch. It's not shared with the agent; it's a human scratchpad.

- Placeholder: `"What are you working on?"`
- Persisted to `~/.indiana/focus.txt` (plain text, one line).
- Saved on blur / Enter. Loaded on panel open.
- Empty by default; no placeholder for the user who leaves it blank.

### 2. Monitored folders

Same as the existing PRD, but with a tighter spec:

- List: each folder shows its basename (or tilde-path) and the live marker count Indiana reports.
- Add `[+]`: opens a native `NSOpenPanel` picking a directory. Sends the path to Indiana over the socket; Indiana adds it to `config.json` and rescans.
- Remove: right-click → "Stop monitoring". Sends to Indiana; Indiana removes from `config.json`.
- Click a folder: copies its compiled bundle to clipboard. Visual feedback: brief checkmark or "Copied" flash.
- Empty state: when no folders are monitored, show a centered "Monitor a folder →" prompt that triggers the add picker.

Count per folder is tallied by Indiana (actions, fixes, questions, etc.) — the menulet just displays the number. Full kind breakdown deferred; a single total is enough for v1.

### 3. Server control

At the bottom, a status line with an explicit start/stop button.

- Running: green dot ● + "Server running" + stop button ⏹.
- Stopped: grey dot ○ + "Server stopped" + start button ▶.
- Starting: spinner + "Starting…" (transient state while sidecar spawns).
- Stop: sends shutdown command over the socket, then waits for socket close. Graceful — finishes in-flight compile, doesn't kill abusively.
- Start: spawns the bundled `indiana serve` sidecar. Same logic as launch-time auto-spawn (check socket first; if a launchd daemon is already alive, just connect).

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
| Server running, 1+ folders | Full layout: focus, folder list with counts, green dot |
| Server running, no folders | Focus field + empty state ("Monitor a folder →") + green dot |
| Server stopped | Focus field, folder list greyed out (counts hidden), grey dot + start button |
| Launch (first ever) | Everything empty. Focus blank, no folders, server stopped. |

---

## What changed from v1 PRD

- Added: FOCUS ON text field with persistence.
- Added: explicit start/stop server control (was implicit launch-time auto-spawn only).
- Tightened: folder list spec — basename display, click-to-copy, right-click-to-remove, single-count display.
- Decided: focus lives in its own file (`focus.txt`), not in config.
