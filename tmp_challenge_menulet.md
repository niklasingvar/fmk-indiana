# Why not Electron for the Indiana menulet?

## The honest answer: Electron would work.

The menulet is a 320px panel with a text input, a folder list, and a status bar. It connects to a Unix socket, copies strings to the clipboard, and spawns a child process. Electron can do all of that. `menubar` (the npm package) makes the tray+panel dance a one-liner. `node:net` talks to the socket. `electron-builder` packages it.

So the question isn't "can Electron do it" — it's "what do we pay for Electron, and is it worth it?"

---

## What Electron costs for this product

### 1. Bundle size
An unsigned Electron `.app` with no app code is ~150 MB (Chromium + Node.js + V8 + Blink). Indiana's entire daemon binary is 302 KB. Shipping a 500× penalty for a panel that shows three numbers is absurd. The Tauri 2 `.app` will be ~4–6 MB (WebKit is on the system; only the Rust binary + HTML/CSS travel).

### 2. Memory
Electron pulls ~150–250 MB RSS at idle for a single window. This menulet sits in the menu bar 24/7 — the user didn't ask for a browser, they asked for a status indicator. Tauri on macOS uses WKWebView, which shares memory with every other WebKit client on the system and idles at ~30–50 MB.

### 3. Two runtimes, zero benefit
The daemon is Rust. The protocol types are Rust. The sidecar management is spawning a Rust binary. With Electron we'd have:
- `indiana` (Rust, the daemon)
- `Indiana.app/Contents/Resources/app/` (Node.js, the menulet)
- Two separate socket client implementations (Node.js `net` + Rust `UnixStream`)

With Tauri we get:
- `indiana` (Rust, the daemon)
- `Indiana.app/Contents/MacOS/indiana` (same Rust binary as sidecar)
- One socket client implementation (shared, or duplicated trivially in the same language)

The Tauri backend speaks the same protocol types. No JSON shape drift between two languages.

### 4. macOS integration is worse, not better
Electron's `LSUIElement` support works, but it fights you on panel behavior — Electron windows are `NSWindow`, not `NSPanel`, and the hide-on-blur / floating-panel behavior requires workarounds. Tauri 2 uses the native `NSWindow` level directly through its Rust backend. The tray icon, the panel positioning, the accessory activation policy — all are thin wrappers over AppKit, not reimplementations.

### 5. The "ecosystem" argument doesn't hold
The Electron argument is usually: "more packages, faster to build." But this menulet has:
- Zero HTTP requests
- Zero rendering complexity (no charts, no rich text)
- Zero authentication
- Zero database

It reads a socket, shows a list, copies text. The "ecosystem" is `net.createConnection` and `clipboard.writeText` — 15 lines of Node.js. The scaffold overhead (electron-builder config, signing, notarization, auto-update wiring) dwarfs the app logic either way. Tauri's scaffold overhead is comparable, and the result is 50× smaller.

---

## When Electron would be the right call

- The team knows Electron and nobody knows Rust/Tauri.
- The panel is complex enough that a web frame is genuinely needed *and* Electron's Chromium renderer provides something WebKit doesn't (e.g., complex CSS, WebGL, devtools integration).
- You're already shipping an Electron app for another product and this is a companion.

None of these apply here. The team already writes Rust. The panel is a list and a text field. The daemon is Rust. The sidecar is Rust. Adding Node.js to the stack introduces a second language, a second package manager, a second build pipeline, and a 500× size penalty — for what?

---

## What Tauri costs in return

| Cost | Reality |
|------|---------|
| Learning Tauri 2 | The Tauri 2 API surface for this app is ~6 commands, a tray icon, and a window config. The learning curve is a few hours. |
| WebKit quirks | `-webkit-appearance`, `backdrop-filter`, and flexbox bugs are real. But this panel has none of those — it's system font, a list, and a border. |
| Smaller ecosystem | The `tauri-plugin-shell` sidecar API exists and is stable. The `tauri-plugin-clipboard` exists. There's no `menubar` equivalent — but the 20 lines of Rust to create a tray + hidden window + toggle-on-click are already in the runbook. |
| Build toolchain | `npm install` + `cargo build` — same as Electron (`npm install` + `electron-builder`), just with `cargo` instead of `node-gyp` rebuilds. |

---

## The real reason, in one sentence

**The menulet is a thin face onto a Rust daemon — adding 150 MB of Chromium and a second language runtime to display three numbers is wrong by two orders of magnitude.**
