---
status: draft
purpose: How Indiana (CLI + daemon) and the Menulet reach users, and how the sidecar is hosted.
approval: pending
---

# DISTRO — distribution

> How [Indiana](indiana/IN_PRD.md) (CLI + server) and the [Menulet](menulet/MENULET_PRD.md) reach users. Steps mirror [PHASES.md](PHASES.md).

## Now (dogfood)
- One binary, multi-mode: `indiana serve` (daemon), `indiana scan`, `indiana copy`, `indiana service install`, `indiana todo`.
- Single static `aarch64-apple-darwin` artifact, no runtime deps.
  - Install by copy to `~/.local/bin/indiana`; `cargo build --release` from source.
  - SQLite is compiled in via `rusqlite`'s `bundled` feature (build-time `cc` step, no system SQLite, still no runtime dep). It backs the repo-local Montmartre todo list only.
  - Daemonize via `launchd` (`indiana service install`); CLI and menulet both talk to the daemon over a Unix domain socket at `~/.indiana/indiana.sock`.
- Menulet: `cargo tauri build` → unsigned `.app`; drag to `/Applications`.
  - Bundles the `indiana` server binary as a Tauri sidecar inside the `.app` bundle. On launch, spawns it as a child if no daemon is already running.
## Now (friend testers) — Homebrew tap, unsigned

Ship the current version to a handful of friends on Apple Silicon Macs:

```sh
brew tap niklasingvar/fmk-indiana
brew install --cask --no-quarantine indiana-menulet   # GUI, bundles the daemon
brew install niklasingvar/fmk-indiana/indiana          # optional standalone CLI
```

- Tag-triggered release: pushing `vX.Y.Z` runs `.github/workflows/release.yml`,
  which builds the CLI tarball + unsigned menulet `.dmg` (aarch64), publishes a
  GitHub release, and bumps the tap (`niklasingvar/homebrew-fmk-indiana`).
- Authoritative formula/cask live in `dist/homebrew/`; the workflow copies them
  into the tap with the per-release `url`/`sha256`/`version` filled in.
- Unsigned: friends pass `--no-quarantine` (or right-click → Open). Signing is the
  next step below.
- Menulet self-contains: bundles `indiana` as a Tauri sidecar. On launch it connects
  to an existing daemon on the Unix socket, else spawns the bundled `indiana serve`.
  If `indiana` is on `PATH` and newer, it prefers that — decoupled upgrades.
- Validate a build locally before tagging with `make dist` (same steps, prints SHA256s).

## Next (public)
- Menulet: code-signed + notarized `.dmg` (Developer ID) so Gatekeeper passes with no
  `--no-quarantine`. Needs an Apple Developer account.
- Intel / universal binaries (current testers are all Apple Silicon).
- Auto-update channel (e.g. Sparkle / Tauri updater).

## Sidecar (Tauri host)
- Binary name: the bundled server is `Indiana.app/Contents/MacOS/indiana` — named `indiana`, not the target triple. Tauri's `tauri.conf.json` sidecar config must match.
- Signing: the sidecar is signed as part of the app bundle with hardened runtime. Unsigned sidecar → notarization fails, so the `indiana` build must be hardened-runtime compatible.
- Process management: the menulet kills only the child it spawned. If a launchd-installed daemon is already running, the menulet connects to it and never kills it.
- PATH detection: a GUI app's PATH is the launchd default (`/usr/bin:/bin:/usr/sbin:/sbin`), not the user's shell PATH. Resolve by checking standard locations — `~/.local/bin`, `/usr/local/bin`, the Homebrew prefix — rather than parsing `.zshrc` / `.zprofile`.
- Lifecycle and socket details: [indiana/IN_DAEMON.md](indiana/IN_DAEMON.md).

## IPC
- Server binds a Unix domain socket at `~/.indiana/indiana.sock`.
- Clients (CLI, menulet) speak a minimal protocol over the socket — JSON or bincode.
- No HTTP, no port conflicts, no auth beyond file permissions. Local-only by construction.

## Install flow by audience
| Who | How |
|-----|-----|
| Dogfood | `cargo build --release` + manual copy. |
| CLI-native early user | `brew install niklasingvar/fmk-indiana/indiana`. `indiana service install` to daemonize. |
| GUI early user | `brew install --cask --no-quarantine indiana-menulet`. Menulet self-contains — no terminal needed. |
| Both | Cask for the menulet + formula for the CLI. Menulet detects the `PATH` binary, prefers it. |
## Open questions
- Signing identity / Apple Developer account ownership.
