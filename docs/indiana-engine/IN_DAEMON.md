---
status: draft
purpose: Specify daemon lifecycle, the config it owns, socket binding, and crash recovery.
max_lines: 50
approval: pending
---

# IN_DAEMON — lifecycle and state

> The long-lived core process. Shape: [IN_ARCHITECTURE.md](IN_ARCHITECTURE.md). Distribution: [../DISTRO.md](../DISTRO.md). Invariants: [IN_PRINCIPLES.md](IN_PRINCIPLES.md).

## What it is
- One daemon (`indiana serve`) holds the scan, index, counts, and compiled payload in memory.
- Faces (MCP, CLI, menulet) are clients over the Unix socket `~/.indiana/indiana.sock`.

## Config it owns
- The monitored-folders list lives in `~/.indiana/config.json`.
- This is input, not derived state — it is not in any markdown and not rebuildable from source.
- The one legitimate non-source state ([IN_PRINCIPLES.md](IN_PRINCIPLES.md): source is truth applies to the index, not to user config).
- The index stays throwaway; config is user choice and persists across restarts.
- Empty config monitors nothing — a folder must be selected before the daemon scans anything.

## Folder selection
- `indiana add <path>` selects a folder and initializes repo-local command templates ([IN_FOLDER.md](IN_FOLDER.md)).
- Against a running daemon it is a live command: the daemon persists `config.json`, starts watching the folder, and rebuilds the index immediately (no restart).
- With no daemon running, `add` writes `config.json` and initializes `.indiana/`; the next `serve` picks it up.
- `indiana serve <path>` initializes `.indiana/` for an explicit monitored root.

## Socket binding
- One process binds the socket. A second `indiana serve` must fail to bind with a clean error, not clobber.
- On startup, handle a stale socket from a crashed daemon: try to connect first.
  - Connect succeeds → a daemon is alive; exit with a clear "already running".
  - Connection refused → stale file; unlink, then bind.
- No auth beyond filesystem permissions — local-only by construction.

## Crash recovery
- launchd-installed daemon: `KeepAlive` restarts it on crash.
- Menulet-spawned daemon has no launchd guard. The menulet watches the socket; on close it respawns the child.
- Recovery is cheap: the index rebuilds from a full scan on startup ([IN_SCAN.md](IN_SCAN.md)).

## Client disconnect
- State is in memory. A client that disconnects and reconnects gets current state — no replay, no session log.
- An in-flight operation (a copy compile) completes regardless of whether the client is still attached.

## Open
- Whether a `remove`/stop-monitoring command mirrors `add` (live unwatch + rebuild).
- Whether external edits to `config.json` (not via `add`) hot-reload, or require a restart.
