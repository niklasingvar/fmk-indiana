---
status: draft
purpose: Casablanca forward-looking roadmap — the MVP daily loop and artifact types in order of attack.
approval: pending
---

# CASABLANCA — Roadmap

> What is built today: [CASABLANCA_PRD.md](CASABLANCA_PRD.md). Decisions and constraints: [CASABLANCA_ARCHITECTURE.md](CASABLANCA_ARCHITECTURE.md).

## MVP — the daily loop
- Inline rich markdown editing of repo files, round-trip byte-stable for what the editor doesn't touch:
  - YAML frontmatter preserved as an opaque block (every repo doc carries one).
  - `::` marker lines survive open → edit elsewhere → autosave unchanged.
- `Copy all` button: runs `indiana copy` for the open vault; payload lands on the clipboard; paste into any coding agent.
- Detail lives in [crates/casablanca/TASKS.md](../../crates/casablanca/TASKS.md).

## File-tree operations
The tree is a projection of a vault, and all mutations use one entry-operation
boundary in the main process. The first slice is deliberately small but sets
the vocabulary and safety rules for the rest of this domain.

1. **Delete entry — built first**
   - Delete visible files and folders through a typed `entry:delete` IPC call.
   - Validate vault-relative paths and protect the synthetic vault root.
   - Confirm in the tree UI and move the target to OS Trash, including recursive folder contents.
   - Refresh tree/Git state and settle autosave before removing an open note.
2. **Create and rename**
   - Add validated entry names, collision handling, inline rename, and a new-folder affordance.
   - Keep note content operations (`read`, `write`, `create`) separate from entry lifecycle operations.
3. **Move**
   - Add a path-safe `moveEntry(from, to)` operation with invalid-descendant and vault-escape checks.
   - Add drag/drop only after the command behavior is stable.
4. **Undo and conflict handling**
   - Define restore-from-Trash behavior and operation history.
   - Surface failures when an agent or another process changes the target concurrently.
5. **Batch tree ergonomics**
   - Add reveal-in-Finder, multi-selection, batched operations, and filtering after single-entry semantics are proven.

## Artifact types, in order of attack
1. Documents — write in the rendered view; no edit/preview split. (built — see PRD)
2. Presentations — rendered decks with annotation boxes. (not yet built)
3. Code — raw, with inline commands. (not yet built)
4. Excalidraw canvases — inline diagrams as a first-class artifact type. (not yet built — `ExcalidrawPlugin.tsx` is a stub)
5. Web apps — DOM-observable only. (partially: HTML preview + annotation is built; full web-app artifact type is not)
