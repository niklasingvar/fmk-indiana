---
status: draft
purpose: Entry point for anyone landing on the repo — what Indiana is and how to get it running.
goal: A newcomer reads only this file to install or try Indiana, then follows links into docs/ for depth.
approval: pending
---
# Indiana

> Tag lines with `::` markers while reviewing agent output; Indiana monitors the repo, compiles each marker into a prompt, and exposes the bundle to agents via MCP or to humans via copy. Why and where this goes: [docs/PURPOSE.md](docs/PURPOSE.md).

## Start the Electron editor

### From source

Requires Rust, Node.js, and npm. From the repository root:

```sh
make casablanca
```

This builds the local Indiana binary, installs Casablanca's npm dependencies, and
launches the Electron development app with that binary. No separate Indiana
install is needed. Work in the separate Casablanca window; `http://localhost:5173`
is only the renderer dev server and cannot use the Electron preload bridge. Stop
the app with Ctrl-C in the terminal.

`make casablanca` makes the local binary available for editor actions such as
Copy all. For live monitoring and agent jobs, start the daemon in another terminal:

```sh
make serve
```

On first launch, select a folder to open as a project.

### Installed app

After installing the Casablanca cask, open it from Finder or run:

```sh
open -a Casablanca
```

## Install

### Homebrew (recommended)

```sh
brew tap niklasingvar/fmk-indiana
brew trust niklasingvar/fmk-indiana          # Homebrew 6.x: once per tap
```

Menubar app (bundles the daemon — nothing else to install):

```sh
brew install --cask indiana-menulet
xattr -dr com.apple.quarantine /Applications/Indiana.app
```

Editor (depends on the `indiana` CLI for Copy-all; talks to whichever daemon is running):

```sh
brew install --cask indiana-casablanca
xattr -dr com.apple.quarantine /Applications/Casablanca.app
```

CLI / daemon only, for terminal users:

```sh
brew install niklasingvar/fmk-indiana/indiana
```

> The apps are unsigned, so macOS quarantines them on download. Strip the quarantine
> flag with the `xattr` line after each cask install, or right-click the `.app` → Open
> on first launch. `brew install --cask --no-quarantine` no longer works on Homebrew
> 6.x — see [docs/DISTRO.md](docs/DISTRO.md). Signing is the planned fix.

### From source

```sh
cargo build --release
mkdir -p ~/.local/bin
cp target/release/indiana ~/.local/bin/indiana
```

Add `~/.local/bin` to your `PATH` if it isn't already.

## Test the CLI locally

### Quick dev test (no install)

Use this when you just want to see Indiana run without installing anything.

Terminal 1 — start the server (monitors nothing yet):

```sh
make serve
```

Terminal 2 — make a fixture, select it, then read it:

```sh
make scratch
make add
# This creates tmp/indiana-test/.indiana/ with per-command prompt templates.
make scan
make copy
```

`make add` tells the running server to monitor `tmp/indiana-test`; the server scans it immediately. `make scan` / `make copy` then read the server's live state.

Why this path:

- Uses the release profile, so behavior is close to production.
- Does not copy anything into `~/.local/bin`.
- Uses ignored `tmp/indiana-test` inside this repo, so ID injection cannot touch real notes.

### After install

Verify:

```sh
indiana --version
indiana --help
```

Terminal 1 — start the server:

```sh
indiana serve
```

Terminal 2:

```sh
mkdir -p tmp/indiana-test
printf '%s\n\n%s\n%s\n' \
  'This line needs work ::fix tighten wording' \
  '::action follow up on this' \
  'Next block of context for the action.' \
  > tmp/indiana-test/review.md
indiana add tmp/indiana-test
# This creates tmp/indiana-test/.indiana/ with per-command prompt templates.
indiana scan
indiana copy
```

## Run the menulet

One command builds the daemon, bundles it as the menulet's sidecar, and launches
the menu-bar app:

```sh
make menulet
```

### What happens on start

1. `cargo build --release` — builds the daemon binary.
2. The binary is copied into the menulet's sidecar slot
   (`MENULET/src-tauri/binaries/indiana-aarch64-apple-darwin`).
3. `tauri dev` runs `npm run ui:dev` (Vite serves the panel on
   `localhost:1420`), waits for it, then compiles the Tauri Rust app.
4. The menu-bar tray icon appears; the menulet auto-starts the daemon
   (connect-or-spawn), so the status shows "Server running".

First run compiles the Tauri crate and takes a few minutes; later runs are fast.
The terminal stays attached — Ctrl-C quits the session.

### Use it

- Click the tray icon → panel → "Add folder" to select a directory to monitor
  (run `make scratch` first for a known `tmp/indiana-test` fixture).
- Click a folder to copy its compiled bundle; right-click to remove it.