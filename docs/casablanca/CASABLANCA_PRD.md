---
status: draft
purpose: Specify Casablanca — the editor. Rich inline markdown editing plus visual/presentation features.
approval: pending
---

# CASABLANCA — PRD

> The visual layer of [VISION.md](../../VISION.md). Why the system exists: [PURPOSE.md](../PURPOSE.md). System shape: [ARCHITECTURE.md](../ARCHITECTURE.md). Roadmap: [ACTION_PLAN.md](../../ACTION_PLAN.md) Phase 1.

## What it is
- The editor: open a repo, edit markdown inline as rich text (WYSIWYG, no edit/preview split), see artifacts as what they are — documents as documents, slides as slides.
- Annotating emits ordinary `::` markers into the source file. Casablanca is a face; [Indiana](../indiana/IN_PRD.md) owns the markers.
- A `Copy all` button hands the compiled Indiana payload to the clipboard — the iterate loop without a terminal.

## Architecture decision (revised 2026-07)
- Casablanca is the editor, self-built: Electron + electron-vite + React + Lexical + Tailwind at `crates/casablanca/`. Prototype exists: 3-pane shell, vault folder pane, Lexical WYSIWYG with markdown round-trip, autosave, file watcher.
- Nimbalyst (open source, nimbalyst.com) is vendored at `crates/casablanca/nimbalyst/` as reference only — patterns are borrowed (e.g. its `FlatFileTree`/`FileTreeRow` split), the codebase and name are not.
- Visual/presentation support (inline Excalidraw, rendered decks, annotated views) is a feature set inside Casablanca — not a separate module or product.
- Superseded: "Nimbalyst is the editor, Casablanca builds on it" (earlier 2026-07 framing); Casablanca as a terse agent-output template format (earlier still).

## MVP — the daily loop
- Inline rich markdown editing of repo files, round-trip byte-stable for what the editor doesn't touch:
  - YAML frontmatter preserved as an opaque block (every repo doc carries one).
  - `::` marker lines survive open → edit elsewhere → autosave unchanged.
- `Copy all` button: runs `indiana copy` for the open vault; payload lands on the clipboard; paste into any coding agent.
- Detail lives in [crates/casablanca/TASKS.md](../../crates/casablanca/TASKS.md).

## Artifact types, in order of attack
1. Documents — write in the rendered view; no edit/preview split. (MVP)
2. Presentations — rendered decks with annotation boxes. (feature, after MVP)
3. Code — raw, with inline commands.
4. Excalidraw canvases — inline diagrams as a first-class artifact type.
5. Web apps — DOM-observable only.

## Boundaries
- Casablanca renders and annotates; it never scans, counts, or compiles ([IN_PRINCIPLES.md](../indiana/IN_PRINCIPLES.md): core computes, faces render). `Copy all` delegates to `indiana copy`.
- Viewer config is a local `settings.json` per project — configuration is files in the folder.

## Open questions
- Human edits on rendered views need version handling — parked ([VISION.md](../../VISION.md) non-goals).
- Whether the vendored nimbalyst reference stays in-tree or becomes a link once its patterns are absorbed.
