---
status: draft
purpose: How Indiana (CLI + daemon), the Menulet, and Casablanca reach users, and how the sidecar is hosted.
approval: pending
---

# DISTRO — distribution

> How [Indiana](indiana/IN_PRD.md) (CLI + server), the [Menulet](menulet/MENULET_PRD.md), and [Casablanca](casablanca/CASABLANCA_OVERVIEW.md) reach users. Steps mirror [ACTION_PLAN.md](../ACTION_PLAN.md).

## Now (dogfood)
- One binary, multi-mode: `indiana serve` (daemon), `indiana scan`, `indiana copy`, `indiana service install`, `indiana todo`.
- Single static `aarch64-apple-darwin` artifact, no runtime deps.
  - Install by copy to `~/.local/bin/indiana`; `cargo build --release` from source.
  - SQLite is compiled in via `rusqlite`'s `bundled` feature (build-time `cc` step, no system SQLite, still no runtime dep). It backs the repo-local Chief of Staff todo list only.
  - Daemonize via `launchd` (`indiana service install`); CLI and menulet both talk to the daemon over a Unix domain socket at `~/.indiana/indiana.sock`.
- Menulet: `cargo tauri build` → unsigned `.app`; drag to `/Applications`.
  - Bundles the `indiana` server binary as a Tauri sidecar inside the `.app` bundle. On launch, spawns it as a child if no daemon is already running.

## Now (friend testers) — Homebrew tap, unsigned

Ship the current version to a handful of friends on Apple Silicon Macs:

```sh
brew tap niklasingvar/fmk-indiana
brew trust niklasingvar/fmk-indiana                     # Homebrew 6.x requires this once per tap
brew install --cask indiana-menulet                     # menubar GUI, bundles the daemon
xattr -dr com.apple.quarantine /Applications/Indiana.app  # --no-quarantine is broken on 6.x, see note below
brew install --cask indiana-casablanca                  # editor GUI (depends on the indiana formula for Copy-all)
xattr -dr com.apple.quarantine /Applications/Casablanca.app
brew install niklasingvar/fmk-indiana/indiana           # optional standalone CLI
```

- Tag-triggered release: pushing `vX.Y.Z` runs `.github/workflows/release.yml`,
  which builds the CLI tarball + unsigned menulet `.dmg` + unsigned Casablanca `.dmg`
  (aarch64), publishes a GitHub release, and bumps the tap (`niklasingvar/homebrew-fmk-indiana`).
- Authoritative formula/cask live in `dist/homebrew/`; the workflow copies them
  into the tap with the per-release `url`/`sha256`/`version` filled in.
- Unsigned: friends strip quarantine with `xattr -dr com.apple.quarantine /Applications/<App>.app`
  (or right-click → Open). `brew install --cask --no-quarantine` is broken as of
  Homebrew 6.0.6 (dev build 6.0.6-46-gf2dadbf) — `--no-quarantine` was dropped as a CLI switch
  in favor of `HOMEBREW_CASK_OPTS`, but `cmd/install.rb`'s cask install path never reads that
  env var, so quarantine is force-applied regardless. Signing is the next step below and
  removes this whole workaround.
- Menulet self-contains: bundles `indiana` as a Tauri sidecar. On launch it connects
  to an existing daemon on the Unix socket, else spawns the bundled `indiana serve`.
  If `indiana` is on `PATH` and newer, it prefers that — decoupled upgrades.
- Casablanca does NOT bundle the daemon. The cask `depends_on` the `indiana` formula,
  so `brew install --cask indiana-casablanca` pulls the CLI; "Copy all" shells out to
  it. Start the daemon separately — `indiana serve`, `indiana service install`, or
  open the menulet. One daemon, all faces (ACTION_PLAN Phase 2).
- Auto-run needs Node: the `indiana` formula `depends_on "node"`, and the daemon
  launches Claude Code's ACP adapter via `npx -y @zed-industries/claude-code-acp`
  (fetched + cached on first dispatch — nothing pinned; IN_AUTORUN.md). This covers
  CLI users and Casablanca (its cask depends on the formula). Gap: a menulet-only
  install bundles `indiana` but not the formula, so those users need Node separately
  before auto-run works — fold Node into the menulet cask when that path matters.
  Auto-run is off by default (`config.auto_run`), so this changes nothing until opted in.
- Validate a build locally before tagging with `make dist` (same steps, prints SHA256s).

### One-time setup: HOMEBREW_TAP_TOKEN
- A fine-grained GitHub PAT, stored as the `HOMEBREW_TAP_TOKEN` secret on
  `niklasingvar/fmk-indiana` (`gh secret set HOMEBREW_TAP_TOKEN -R niklasingvar/fmk-indiana`).
- Required scope: Contents: Read and write on `niklasingvar/homebrew-fmk-indiana`. That's the
  only repo the release workflow writes to (`.github/workflows/release.yml`, "Bump Homebrew
  tap" step clones + pushes there). Granting the token access to `fmk-indiana` too is not
  needed but harmless.
- Fine-grained PATs expire. Renew before expiry at
  github.com/settings/personal-access-tokens, then re-run `gh secret set` with the new value —
  otherwise the tap-bump step 403s and the release skill's step 8 catches it (tap still shows
  old checksums).
- If a release run 403s on this step, mint a new PAT first (check expiry on the current one),
  then `gh secret set`, then `gh run rerun --failed <id>`.
- Keep exactly one live PAT for this purpose. Delete stale/unused fine-grained PATs from the
  GitHub settings page once confirmed superseded, so a future 403 doesn't turn into "which
  token is even wired in."

## Next (public)
- Menulet + Casablanca: code-signed + notarized `.dmg` (Developer ID) so Gatekeeper passes
  without the `xattr` workaround. Needs an Apple Developer account.
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
| Menulet GUI | `brew install --cask indiana-menulet` then `xattr -dr com.apple.quarantine /Applications/Indiana.app`. Menulet self-contains — no terminal needed. |
| Casablanca GUI | `brew install --cask indiana-casablanca` then `xattr -dr com.apple.quarantine /Applications/Casablanca.app`. Cask pulls the `indiana` CLI (Copy-all needs it); start the daemon with `indiana serve` or open the menulet. |
| Both GUIs | Both casks. Menulet self-contains its daemon; Casablanca finds the `indiana` CLI the cask pulled. |
## Open questions
- Signing identity / Apple Developer account ownership.
