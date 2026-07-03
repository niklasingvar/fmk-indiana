---
status: draft
purpose: Plan to replace the menulet tray icon with "::" text.
approval: pending
---

# MENULET_LOGO — change tray icon to "::"

> The tray icon loaded from `menulet/src-tauri/icons/tray.png` (line 42 of
> `main.rs`) is a placeholder per M12.1.1. Replace with `::` — the project's
> core symbol (see [COMMANDS.md](../indiana/IN_COMMANDS.md)).

## Current state

- `menulet/src-tauri/icons/tray.png`: 32×32 RGBA PNG, loaded via
  `Image::from_bytes(include_bytes!("../icons/tray.png"))`.
- `icon_as_template(true)` — macOS treats the image as an alpha mask (black →
  system menu-bar color, transparent → background).
- No other code references `tray.png`.

## Options

### A — Replace PNG file only
Replace `tray.png` with a new 32×32 monochrome PNG of "::" (black glyph on
transparent background). Zero code changes.

- Pro: no code changes, no risk of breaking the icon pipeline.
- Con: external asset still exists; "::" at 32×32 may not render sharply
  without careful hinting; updating the glyph needs an image editor.

### B — Generate at runtime in Rust
Drop `tray.png` entirely. Build a 32×32 RGBA pixel array in `setup()` via
`Image::new_rgba(width, height, &pixels)` (or equivalent Tauri 2 API), then
pass it to `TrayIconBuilder`.

- Pro: no external asset; the logo is self-documenting in code; trivial to
  tweak spacing/size.
- Con: requires knowing the exact Tauri 2 `Image` constructor signature
  (likely `Image::new_rgba(u32, u32, Vec<u8>)` or `Image::new(&[u8], u32,
  u32)`); a hand-built bitmap must be verified visually.

### C — Generate PNG at build time (build.rs)
Add `image` as a build dependency. A `build.rs` renders "::" with
`imageproc`/`rusttype` into a PNG, written to `OUT_DIR`, then
`include_bytes!`'d at compile time.

- Pro: crisp text via proper font rasterization.
- Con: two new build dependencies (`image`, `rusttype` or `ab_glyph`);
  slower builds; overengineered for two characters.

## Recommended: B (runtime bitmap)

Reasons:
- No new dependencies, no external files, no build script.
- The "::" glyph is two vertical pairs of dots — trivial to express as a
  32×32 pixel array (roughly 40 lines of Rust).
- Aligns with "simplicity first": dead code removal (tray.png goes away),
  single file touched (`main.rs`), no new crates.
- The glyph lives in code, where it's easy to adjust and obvious to future
  maintainers.

Option A is the fallback if the Tauri 2 `Image` runtime constructor is
unavailable or the visual result is unacceptable. But a hand-tuned 32×32
bitmap of two dots should render cleanly.

## Steps

1. **Verify Tauri 2 Image constructor** — confirm `Image::new_rgba(u32, u32, Vec<u8>)` (or the exact equivalent) compiles with the pinned Tauri 2 version.
2. **Build the pixel array** — hand-craft a 32×32 RGBA bitmap of `::`.
   Template mask: black (`[0, 0, 0, 255]`) where the glyph draws, transparent
   (`[0, 0, 0, 0]`) elsewhere. macOS tints the black parts to the menu bar
   color.
3. **Replace icon loading in main.rs** — swap `Image::from_bytes(include_bytes!(…))` with `Image::new_rgba(32, 32, pixels)`.
4. **Remove dead asset** — delete `menulet/src-tauri/icons/tray.png`.
5. **Verify visually** — `make menulet` → launch `Indiana.app` → the menu bar shows `::` as the icon, clean at both regular and Retina resolutions, light and dark menu bars.

## Verification

| Gate | What proves it |
|------|---------------|
| Compiles | `cd menulet && npx tauri build` produces unsigned `.app` |
| Icon renders | Launch `.app` — `::` visible in menu bar, light + dark menu bar |
| No regressions | Click toggles panel, blur hides it, all M12.6.2 manual steps still pass |
| Dead code gone | `tray.png` removed; `grep -r "tray.png" menulet/src-tauri/src/` returns nothing |

## Notes

- `icon_as_template(true)` stays — we're replacing the image, not the
  rendering mode.
- The app bundle icons (`icon.icns`, `32x32.png`, etc.) are NOT changed —
  those are the `.app` bundle icon (Finder), not the menu bar tray icon.
  Replacing those is out of scope.
- If the Tauri 2 `Image` runtime constructor doesn't accept raw RGBA directly,
  fall back to option A (new PNG, no code change).
