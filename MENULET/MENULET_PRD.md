# MENULET — PRD

> A UI representation of [Indiana](INDIANA/IN_PRD.md). It shows; it does not compute. Markers: [COMMANDS.md](INDIANA/IN_COMMANDS.md).

## What it is
- A macOS menulet (menu-bar item, no Dock icon).
- A thin view onto the Indiana server — connects over a Unix domain socket at `~/.indiana/indiana.sock`.
- Shows which folders Indiana monitors; one click to copy a folder's compiled bundle.

## The sign (menu-bar icon)
- Template icon, always present.
- Click → dropdown panel under the icon, right-aligned; hide on blur.

## Panel contents
- Monitored folders — list; add / remove a folder (registers it with Indiana).
- Per folder: whatever Indiana reports (e.g. tallies by kind) — displayed, not computed here.
- Copy button per folder → puts Indiana's compiled bundle on the clipboard.
- Empty state: "Monitor a folder" picker.

## Behavior
- All data comes from the Indiana core. The menulet never scans or counts.
- Updates as Indiana reports changes over the socket.
- Local only; no accounts, no sync.
- On launch: checks for a running `indiana serve` daemon on the Unix socket. If alive, connects. If not, spawns the bundled binary as a child process.

## Stack
- Tauri 2 (Rust + web UI), accessory activation policy.
- Bundles the `indiana` server binary as a Tauri sidecar inside the `.app` bundle.
- Reuses patterns from `old/adhd-menulet-focus-finder` (tray, NSPanel, hide-on-blur).
## Phase 1 (first deliverable)
- Menulet exists; monitor one folder; list monitored folders.
- Copy lands in a later phase — see [PHASES.md](PHASES.md).
## Open questions
- Copy scope: whole folder vs per-file.
