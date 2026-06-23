---
status: scratch
purpose: Challenge note — why Tauri over Electron for the Indiana menulet.
approval: n/a
---

# Why not a simple Electron app?

> Scratch. The instinct ("Electron is the simple one") is right in general and wrong for *this* codebase. The reason is what already exists around the menulet, not Tauri fanboyism.

## The frame
"Simple" depends on the surrounding system. Indiana already decided four things (DISTRO.md, IN_PRINCIPLES):
- Rust core (`indiana_core`), Rust daemon.
- Single static `aarch64-apple-darwin` binary, no runtime deps.
- The menulet bundles that `indiana` binary as a sidecar.
- The menulet shows, never computes — a thin face over a Unix socket.

Tauri lines up 1:1 with all four. Electron fights all four. So here, Electron is the *more* complex choice dressed as the simple one.

## Where Electron loses against this codebase
- Two runtimes vs one. The backend glue + sidecar are already Rust. Tauri's backend *is* Rust — same language, sidecar is first-class (`app.shell().sidecar("indiana")`). Electron adds a Node runtime whose only job is to spawn a Rust child via `child_process` and proxy a socket. You now maintain JS/Node *and* Rust for a face that does almost nothing.
- Footprint. Electron ships its own Chromium + Node: ~150–200 MB app, ~100 MB+ idle RAM. Tauri uses the system WKWebView: single-digit-MB app, tens-of-MB idle. For an always-present menu-bar accessory that "shows, never computes," a heavyweight idle process is the wrong profile and contradicts "single static artifact, no runtime deps."
- Sidecar story. Bundling and signing a Rust binary inside the app is a documented Tauri path (`externalBin`, signed as part of the bundle). In Electron it's a manual `extraResources` + spawn + per-arch packaging chore you own end to end.
- One toolchain. `cargo` + `tauri` builds the whole thing. Electron means npm/Node *plus* a separate Rust build for the sidecar — two dependency trees, two update cadences.

## Where Electron actually wins (steelman)
- Faster first prototype if the team lives in JS and not Rust.
- Bigger ecosystem — e.g. the `menubar` npm package gives tray + panel out of the box.
- More mature macOS packaging/notarization tooling (`electron-builder`) and far more StackOverflow coverage.

These are real. They'd decide it if Indiana were a JS project with no Rust and no size budget. It isn't.

## The honest tie-breakers (not Tauri wins)
- True menu-bar `NSPanel` (non-activating, hide-on-blur) needs native work in *both*. Neither is free. Plain Tauri window approximates it; may need `tauri-nspanel`. Electron approximates it too; true NSPanel needs a native module.
- Signing + notarization is required for both. Electron's tooling is slightly more turnkey today.

## Verdict
- Pick Tauri because the project is already Rust + single-binary + sidecar + thin-face. Tauri is the low-friction path *given those*; Electron would bolt a second runtime and ~50× the bytes onto a face that proxies a socket.
- Reverse only if the architecture changes (no Rust core, web-heavy UI, size irrelevant). It hasn't.
