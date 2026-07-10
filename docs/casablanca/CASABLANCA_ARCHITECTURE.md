---
status: draft
purpose: Casablanca architecture decisions and ownership boundaries.
approval: pending
---

# CASABLANCA — Architecture

> Product overview: [CASABLANCA_OVERVIEW.md](CASABLANCA_OVERVIEW.md). Implemented feature inventory: [CASABLANCA_PRD.md](CASABLANCA_PRD.md).

## Architecture decision (revised 2026-07)
- Casablanca is the editor, self-built: Electron + electron-vite + React + Lexical + Tailwind at `crates/casablanca/`. Prototype exists: 3-pane shell, vault folder pane, Lexical WYSIWYG with markdown round-trip, autosave, file watcher.
- Nimbalyst (open source, nimbalyst.com) is vendored at `crates/casablanca/nimbalyst/` as reference only — patterns are borrowed (e.g. its `FlatFileTree`/`FileTreeRow` split), the codebase and name are not.
- Visual/presentation support (inline Excalidraw, rendered decks, annotated views) is a feature set inside Casablanca — not a separate module or product.
- Superseded: "Nimbalyst is the editor, Casablanca builds on it" (earlier 2026-07 framing); Casablanca as a terse agent-output template format (earlier still).

## Boundaries
- Casablanca renders and annotates; it never scans, counts, or compiles ([IN_PRINCIPLES.md](../indiana/IN_PRINCIPLES.md): core computes, faces render). `Copy all` delegates to `indiana copy`.
- Viewer config is a local `settings.json` per project — configuration is files in the folder.

## File-tree operations
- The tree is a read projection of the vault; `TreeNode` is not an authorization to mutate an arbitrary path.
- Renderer actions address vault entries with vault-relative POSIX paths and cross the typed preload bridge. The renderer never touches the filesystem.
- Entry mutations live in a main-process operation boundary separate from note authoring. `notes.read/write/create` handle note content; entry operations handle file and folder lifecycle.
- Every operation resolves and validates its target beneath the vault root. The synthetic vault root cannot be mutated.
- Delete is Trash-backed through Electron's `shell.trashItem`, so recursive folder deletion remains reversible and does not become a permanent filesystem primitive.
- After a mutation, the main process refreshes the tree and Git projection. The renderer clears an open note before its path or an ancestor is removed, and settles autosave first so an in-flight write cannot recreate a deleted entry.
- Future rename, create-folder, move, and restore operations extend this boundary rather than adding note-specific IPC handlers.
