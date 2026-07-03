---
status: superseded
purpose: Archived plan — .indiana meta-model restructure (executed).
approval: pending
---

# Plan: Restructure `.indiana/` into a meta-model layout

## Context

Today the repo-local command prompts live at `.indiana/indianas/<cmd>/prompt.md`
(keep, note, love, todo, question, action, elaborate, fix, hate — plus two empty
`context`/`focus` dirs). But the Rust tool reads templates from
`.indiana/<cmd>/prompt.md` (no `indianas/` segment) — see
`crates/core/src/templates.rs::template_path` and `init_folder_indiana`. So the
parked `.indiana/indianas/` files are **not** consumed by the tool; the tool
falls back to embedded `crates/core/prompts.toml`.

The user wants `.indiana/` to become a small meta-model workspace:

- `.indiana/indianas/` — the command templates, and the **source of truth the
  tool copies into the compiled payload**. Editing a word in
  `.indiana/indianas/keep/prompt.md` must flow into the payload.
- `.indiana/context-model/` — empty for now (current state, direction, rules).
- `.indiana/montmartre/` — project management: `README.md` + `actions.md`,
  `notes.md`, `focus.md`.

This means the Rust template lookup/init must be rewired to use the
`.indiana/indianas/<cmd>/prompt.md` location so the existing (and future edits to
the) command files actually drive the payload.

## Changes

### 1. Rewire the tool to read/write `.indiana/indianas/<cmd>/prompt.md`

`crates/core/src/templates.rs`:
- `template_path()` (read): change
  `root.join(".indiana").join(command).join("prompt.md")`
  → `root.join(".indiana").join("indianas").join(command).join("prompt.md")`.
- `init_folder_indiana()` (write): change
  `root.join(".indiana").join(spec.long)`
  → `root.join(".indiana").join("indianas").join(spec.long)`.

`crates/core/src/walk.rs` — no change. `PRUNE = [".indiana", ".git"]` already
excludes the whole `.indiana/` tree from scanning, so `indianas/`,
`context-model/`, and `montmartre/` are all kept out of review content.

### 2. Update tests that hardcode the old path

Replace `.indiana/<cmd>/prompt.md` with `.indiana/indianas/<cmd>/prompt.md` in:
- `crates/indiana/tests/daemon.rs:258`
- `crates/indiana/tests/mcp.rs:97`
- `crates/indiana/tests/cli.rs:74, 99, 128, 144, 145`

### 3. Scaffold the new repo-local folders

- `.indiana/indianas/` — already exists with the 9 command folders. Leave as-is.
  (The empty `context`/`focus` dirs under it are not real markers and are ignored
  by the tool; left untouched — not part of this change.)
- `.indiana/context-model/` — create, keep empty. Add a `.gitkeep` so git tracks
  the empty directory.
- `.indiana/montmartre/` — create with:
  - `README.md` — one-line description: montmartre is project-management
    (repo actions, notes, focus).
  - `actions.md`, `notes.md`, `focus.md` — each seeded with a one-line `#`
    heading (per "seed with one-line headers").

### 4. Update docs to the new path

Change `.indiana/<command>/` → `.indiana/indianas/<command>/` where the layout
is documented:
- `docs/indiana/IN_FOLDER.md` — the Layout block and the bullet list
  (`.indiana/fix/prompt.md` → `.indiana/indianas/fix/prompt.md`, etc.). Add a
  short note that `.indiana/` also holds sibling `context-model/` and
  `montmartre/` folders.
- `docs/indiana/IN_PRD.md:18`
- `docs/menulet/MENULET_UI.md:39`

## Decisions / assumptions

- `context-model/` and `montmartre/` are created **in this repo only**. The tool's
  `init_folder_indiana` is NOT extended to scaffold them into every monitored
  root — it continues to manage only the `indianas/` command templates. (Flag if
  you want them auto-created in every monitored root.)
- Embedded `prompts.toml` remains the default when a root has no
  `.indiana/indianas/<cmd>/prompt.md`; the folder file wins when present
  (unchanged precedence, just relocated path).

## Verification

1. `cargo test -p indiana` and `cargo test -p indiana-core` — the updated path
   tests pass.
2. End-to-end payload check (proves "edit a word → part of payload"):
   - Edit a distinctive word into `.indiana/indianas/keep/prompt.md` body.
   - Run a compile over a markdown file containing a `::keep` marker for this
     root (via the CLI compile path / daemon) and confirm the edited wording
     appears in the compiled payload, not the embedded default.
3. `indiana add <tmpdir>` (or `templates refresh`) creates
   `<tmpdir>/.indiana/indianas/<cmd>/prompt.md` files (new location), and
   re-running leaves existing files byte-identical.
4. Confirm `.indiana/context-model/` and `.indiana/montmartre/` exist with the
   seeded files and are excluded from scanning.
