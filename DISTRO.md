---
status: draft
purpose: How Indiana (CLI + daemon) and the Menulet reach users, and how the sidecar is hosted.
approval: pending
---

# DISTRO — distribution

> How [Indiana](INDIANA/IN_PRD.md) (CLI + server) and the [Menulet](MENULET_PRD.md) reach users. Steps mirror [PHASES.md](PHASES.md).

## Now (dogfood)
- One binary, multi-mode: `indiana serve` (daemon), `indiana scan`, `indiana copy`, `indiana service install`.
- Single static `aarch64-apple-darwin` artifact, no runtime deps.
  - Install by copy to `~/.local/bin/indiana`; `cargo build --release` from source.
  - Daemonize via `launchd` (`indiana service install`); CLI and menulet both talk to the daemon over a Unix domain socket at `~/.indiana/indiana.sock`.
- Menulet: `cargo tauri build` → unsigned `.app`; drag to `/Applications`.
  - Bundles the `indiana` server binary as a Tauri sidecar inside the `.app` bundle. On launch, spawns it as a child if no daemon is already running.
## Next (early users)
- CLI: Homebrew tap (`brew install niklas/indiana/indiana`).
- Menulet: code-signed + notarized `.dmg` (Developer ID) so Gatekeeper passes.
  - Self-contained: bundles the `indiana` server binary as a Tauri sidecar. No separate server install required.
  - On launch, checks for an existing daemon on the Unix socket. If alive, connects; if not, spawns the bundled `indiana serve`.
  - If `indiana` is on `PATH` and newer, prefers it over the bundled copy — decoupled upgrades for power users.
- Auto-update channel (e.g. Sparkle / Tauri updater).
- Versioned releases on GitHub — one `.dmg` (menulet) + one tarball (CLI binary).

## Sidecar (Tauri host)
- Binary name: the bundled server is `Indiana.app/Contents/MacOS/indiana` — named `indiana`, not the target triple. Tauri's `tauri.conf.json` sidecar config must match.
- Signing: the sidecar is signed as part of the app bundle with hardened runtime. Unsigned sidecar → notarization fails, so the `indiana` build must be hardened-runtime compatible.
- Process management: the menulet kills only the child it spawned. If a launchd-installed daemon is already running, the menulet connects to it and never kills it.
- PATH detection: a GUI app's PATH is the launchd default (`/usr/bin:/bin:/usr/sbin:/sbin`), not the user's shell PATH. Resolve by checking standard locations — `~/.local/bin`, `/usr/local/bin`, the Homebrew prefix — rather than parsing `.zshrc` / `.zprofile`.
- Lifecycle and socket details: [INDIANA/IN_DAEMON.md](INDIANA/IN_DAEMON.md).

## IPC
- Server binds a Unix domain socket at `~/.indiana/indiana.sock`.
- Clients (CLI, menulet) speak a minimal protocol over the socket — JSON or bincode.
- No HTTP, no port conflicts, no auth beyond file permissions. Local-only by construction.

## Install flow by audience
| Who | How |
|-----|-----|
| Dogfood | `cargo build --release` + manual copy. |
| CLI-native early user | `brew install niklas/indiana/indiana`. `indiana service install` to daemonize. |
| GUI early user | Download `.dmg`, drag to `/Applications`. Menulet self-contains — no terminal needed. |
| Both | Homebrew for CLI + `.dmg` for menulet. Menulet detects `PATH` binary, prefers it. |
## Open questions
- Signing identity / Apple Developer account ownership.
