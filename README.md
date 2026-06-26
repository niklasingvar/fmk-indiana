---
status: draft
purpose: Quick local testing steps for Indiana.
approval: pending
---

# Indiana

## Test locally

## Quick dev test

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

## Install

### Homebrew (recommended)

```sh
brew tap niklasingvar/fmk-indiana
```

Menubar app (bundles the daemon — nothing else to install):
```sh
brew install --cask --no-quarantine indiana-menulet
```

CLI / daemon only, for terminal users:
```sh
brew install niklasingvar/fmk-indiana/indiana
```

> The app is **unsigned**, so `--no-quarantine` is required — it tells macOS
> Gatekeeper to trust the download. If you ever install without it and macOS
> blocks the app, either right-click `Indiana.app` → **Open**, or run:
> ```sh
> xattr -dr com.apple.quarantine /Applications/Indiana.app
> ```

### From source
```sh
cargo build --release
mkdir -p ~/.local/bin
cp target/release/indiana ~/.local/bin/indiana
```
Add `~/.local/bin` to your `PATH` if it isn't already.

## Test locally

After installing, verify:
```sh
indiana --version
indiana --help
```

### Scratch test
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
