---
status: draft
purpose: Specify the macOS menulet face for Indiana.
approval: pending
---

# MENULET — PRD

> A UI representation of [Indiana](../indiana/IN_PRD.md). It shows; it does not compute. Markers: [COMMANDS.md](../indiana/IN_COMMANDS.md).

## What it is
- A macOS menulet (menu-bar item, no Dock icon).
- A thin view onto the Indiana server — connects over a Unix domain socket at `~/.indiana/indiana.sock`.
- Shows which folders Indiana monitors; one click to copy a folder's compiled bundle.

## The sign (menu-bar icon)
- Template icon, always present.
- Click → dropdown panel anchored under the icon (positioned from the tray
  event rect; correct across displays and Spaces). Hide on blur.
- Panel is a non-activating NSPanel: never steals focus, appears on the
  active Space (incl. over fullscreen apps) with no Space-switch jump.

## Look
- Boxed TUI / minimalist: monospace, 1px sharp border, no frost.
- Theme switch via cogwheel: light / dark / system; persisted (localStorage).

## Panel contents
- Top: server status + start/stop (start/stop only when the menulet owns
  the daemon).
- Add folder action, then a divider, then a "monitoring" header carrying
  the theme cogwheel.
- Monitored folders — list; add / remove a folder (registers it with Indiana).
- Per folder row: name · count (`N ::`) · copy. Copy puts Indiana's compiled
  bundle on the clipboard; right-click removes the folder.
- Numeric batch rows sit below their folder, sorted by label: `-1 · 4 commands · run · copy`. Counts and grouping come from the daemon.
- Empty state: "monitor a folder" picker.

## Behavior
- All data comes from the Indiana core. The menulet never scans or counts.
- Updates as Indiana reports changes over the socket.
- Batch Run asks the daemon for one ACP turn covering that repo's group. Batch Copy asks for the same filtered compiled payload.
- While the panel is focused, `Ctrl+1` through `Ctrl+9` run the topmost visible matching group; adding Alt copies it instead. Groups above 9 use their row actions.
- Local only; no accounts, no sync.
- On launch: checks for a running `indiana serve` daemon on the Unix socket. If alive, connects. If not, spawns the bundled binary as a child process.

## Stack
- Tauri 2 (Rust + web UI), accessory activation policy.
- Bundles the `indiana` server binary as a Tauri sidecar inside the `.app` bundle.
- `tauri-nspanel` (v2 branch) converts the window to a non-activating NSPanel
  for tray-anchored, multi-Space, focus-preserving presentation.
## Phase 1 (first deliverable)
- Menulet exists; monitor one folder; list monitored folders.
- Copy lands in a later phase — see [PHASES.md](../PHASES.md).
## Open questions
- Copy scope: whole folder vs per-file.
