---
status: draft
purpose: Casablanca dev README — stack, layout, run instructions, build phases.
approval: pending
---

# Casablanca

A minimal Electron note editor: a folder pane on the left, a WYSIWYG rich
editor in the middle, and inline Excalidraw diagrams. Nothing else.

## Stack

- Electron + electron-vite + React + TypeScript
- Lexical rich-text editor with markdown round-trip
- @excalidraw/excalidraw (inline diagrams — phase 4)
- Tailwind for the chrome; plain markdown files on disk for storage

## Layout

```
src/
  shared/        domain model + IPC contract (single source of truth)
  main/          Electron main: app lifecycle, filesystem gateway, watcher
    lib/         vault fs operations, persisted config
  preload/       contextBridge — the renderer's only gateway to the system
  renderer/
    src/
      app/             3-pane shell + empty state
      folder-pane/     vault tree + new note
      editor/          Lexical editor + plugins (markdown, excalidraw)
      storage/         useVault hook (vault/tree/note orchestration)
```

## Develop

```bash
npm install
npm run dev
```

On first launch, choose a vault folder (any directory; markdown notes live
there). Notes autosave as you type.

`npm run dev` opens a separate Electron window — use that. Do not open the
dev-server URL (`http://localhost:5173`/`5174`) in a browser: it serves only
the renderer, has no `window.api`, and will always show the "runs in Electron"
error screen.

## Phases

1. Scaffold + 3-pane shell
2. Folder pane + filesystem reads
3. Lexical WYSIWYG + markdown import/export + autosave
4. Inline Excalidraw (DecoratorNode + fenced-block transformer)
5. Polish: shortcuts, recent files, vault settings, theming
