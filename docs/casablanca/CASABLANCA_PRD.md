---
status: draft
purpose: Casablanca implemented-feature inventory — one row per feature, grounded in the code.
approval: pending
---

# CASABLANCA — PRD

> What the editor is: [CASABLANCA_OVERVIEW.md](CASABLANCA_OVERVIEW.md). Decisions and boundaries: [CASABLANCA_ARCHITECTURE.md](CASABLANCA_ARCHITECTURE.md). Forward-looking roadmap: [CASABLANCA_ROADMAP.md](CASABLANCA_ROADMAP.md). Open questions: [CASABLANCA_QUESTIONS.md](CASABLANCA_QUESTIONS.md).

Features actually built today. One row = one feature. Paths are relative to `crates/casablanca/src/`.

## Shell & app
- App shell: folder pane + editor pane — `renderer/src/app/Shell.tsx`
- Vault-unset empty state with folder picker — `renderer/src/app/EmptyState.tsx`
- Preload-bridge-missing guard screen — `renderer/src/App.tsx`
- Renderer error boundary — `renderer/src/app/ErrorBoundary.tsx`
- Theme init before first paint + light/dark toggle — `renderer/src/app/theme.ts`, `renderer/src/editor/EditorPane.tsx`
- Top-bar live agent-process chips, daemon-health dot, and question popover — `renderer/src/app/TopBar.tsx`, `main/lib/indiana.ts`

## Folder pane / file tree
- Flat file tree with click-to-expand/collapse and click-to-open — `renderer/src/folder-pane/FileTree.tsx`, `shared/flatten-tree.ts`
- Collapse state persisted per vault in localStorage — `renderer/src/folder-pane/FileTree.tsx`
- Auto-reveal active file's ancestors — `renderer/src/folder-pane/FileTree.tsx`, `shared/flatten-tree.ts`
- Active-row highlight + roving keyboard focus — `renderer/src/folder-pane/FileTree.tsx`, `renderer/src/folder-pane/FileTreeRow.tsx`
- Keyboard navigation (arrows, Home, End, Enter) + type-ahead name search — `renderer/src/folder-pane/FileTree.tsx`, `renderer/src/folder-pane/tree-keys.ts`
- Scroll focused row into view — `renderer/src/folder-pane/FileTree.tsx`
- Chevron + open/closed folder icons + per-file-type icons — `renderer/src/folder-pane/FileTreeRow.tsx`
- Depth indent + vertical guides + name alignment under folders — `renderer/src/folder-pane/FileTreeRow.tsx`
- Hide `.md` in labels, keep sidecars full — `renderer/src/folder-pane/FileTreeRow.tsx`
- ARIA tree semantics (role=tree, aria-level/expanded/selected) — `renderer/src/folder-pane/FileTree.tsx`, `renderer/src/folder-pane/FileTreeRow.tsx`
- New note at vault root with inline name input — `renderer/src/folder-pane/FolderPane.tsx`
- Delete visible files and folders with confirmation, path validation, and OS Trash — `renderer/src/folder-pane/FileTree.tsx`, `renderer/src/folder-pane/FolderPane.tsx`, `main/lib/file-operations.ts`
- Right-click tree entries to reveal them in Finder — `renderer/src/folder-pane/FileTree.tsx`, `main/lib/file-operations.ts`
- Empty-tree state — `renderer/src/folder-pane/FolderPane.tsx`
- Git status tinting on rows with folder aggregation — `renderer/src/folder-pane/FileTreeRow.tsx`, `main/lib/git.ts`

## Editor
- Lexical WYSIWYG markdown editor, per-note remount — `renderer/src/editor/Editor.tsx`, `renderer/src/editor/EditorPane.tsx`
- Markdown import on mount + export on edit — `renderer/src/editor/plugins/MarkdownPlugin.tsx`
- Headings, blockquotes, ordered/unordered lists, inline & fenced code, bold/italic — `renderer/src/editor/Editor.tsx`
- Markdown links import/export — `renderer/src/editor/Editor.tsx`
- GFM tables import/export — `renderer/src/editor/plugins/TableMarkdownTransformer.ts`
- Code-text link merge (``[`text`](url)``) — `renderer/src/editor/plugins/merge-code-links.ts`
- Indiana `::` marker survival through round-trip — `renderer/src/editor/plugins/MarkdownPlugin.tsx`, `renderer/src/editor/markdown-roundtrip.test.ts`
- Presentation-only highlighting for recognized Indiana marker suffixes; fenced and inline code remain plain; rich clipboard data omits the presentation style — `renderer/src/editor/plugins/MarkerHighlightPlugin.tsx`
- Undo/redo, auto-focus, placeholder — `renderer/src/editor/Editor.tsx`
- Clickable links → vault note or external browser — `renderer/src/editor/Editor.tsx`, `renderer/src/editor/EditorPane.tsx`
- Clickable "code chip" links — `renderer/src/editor/Editor.tsx`
- @-mention vault file links with suggestion list — `renderer/src/editor/plugins/MentionLinkPlugin.tsx`, `renderer/src/editor/EditorPane.tsx`
- Frontmatter preserved as opaque YAML block — `shared/note-serialization.ts`, `renderer/src/storage/useVault.ts`
- Right-side Properties inspector: editable raw YAML frontmatter — `renderer/src/editor/FrontmatterPanel.tsx`, `shared/frontmatter.ts`
- Debounced autosave (500ms) with Saved/Saving status — `renderer/src/storage/useVault.ts`, `renderer/src/editor/EditorPane.tsx`
- Note navigation history with back/forward + ⌘[/⌘] shortcuts — `renderer/src/storage/useVault.ts`, `renderer/src/editor/EditorPane.tsx`
- Active-note path in header — `renderer/src/editor/EditorPane.tsx`
- Copy all → `indiana copy` with inline success/failure status — `renderer/src/editor/EditorPane.tsx`, `main/lib/indiana.ts`
- Per-note history panel: commits touching the note + "Current changes" entry, unified red/green source diff, read-only — `renderer/src/history/HistoryPanel.tsx`, `shared/diff.ts`

## HTML preview & annotations
- HTML notes open in preview iframe (not Lexical) via `vault://` — `renderer/src/preview/HtmlPreview.tsx`
- Manual reload + annotate toggle — `renderer/src/preview/HtmlPreview.tsx`
- Injected annotator: hover/click/select element — `main/preview/annotator.js`, `renderer/src/preview/HtmlPreview.tsx`
- Shared marker composer with command chips, free-text autocomplete, and keyboard submission — `renderer/src/MarkerComposer.tsx`
- Annotation bubble overlay → shared composer writes `::` marker to `.html.md` sidecar — `renderer/src/preview/AnnotationBubble.tsx`, `main/lib/annotations.ts`
- 9 annotation kinds with message contracts — `shared/annotation-line.ts`, `renderer/src/preview/AnnotationBubble.tsx`

## Main / IPC / vault backend
- Electron bootstrap + sandboxed BrowserWindow with contextIsolation — `main/index.ts`
- External URL security boundary (setWindowOpenHandler + will-navigate → openExternal) — `main/index.ts`
- Privileged `vault://` scheme + custom protocol handler — `main/preview/protocol.ts`
- Vault path traversal guard + MIME mapping — `main/preview/resolve-path.ts`
- HTML annotator script injection via protocol — `main/preview/protocol.ts`
- Vault config persistence in userData — `main/lib/config.ts`
- Vault folder chooser dialog — `main/ipc.ts`, `renderer/src/app/EmptyState.tsx`
- Vault tree projection: Markdown, HTML, and JSON files; folders-first, natural case-insensitive sort; skip heavy dirs (.git/node_modules/target/dist/out) — `main/lib/vault.ts`
- Note read/write/create IPC with post-mutation tree+git refresh — `main/ipc.ts`, `main/lib/vault.ts`
- Generic entry-delete IPC with recursive folder support through OS Trash — `main/ipc.ts`, `main/lib/file-operations.ts`, `shared/ipc.ts`, `preload/index.ts`
- New note created with default `# title` body — `main/lib/vault.ts`
- File watcher (chokidar) → tree/git/preview refresh, cleanup on quit — `main/watcher.ts`, `main/index.ts`
- Git working-tree status (porcelain) with folder aggregation — `main/lib/git.ts`
- Auto `git init` + initial snapshot commit for projects without a repo (only git write Casablanca ever does; loop commits belong to the coding agent) — `main/lib/git.ts` (`ensureRepo`), `main/ipc.ts`
- Per-note git log + diff IPC (`git:log`, `git:diff-commit`, `git:diff-head`), untracked files synthesized via `--no-index` — `main/lib/git.ts`, `main/ipc.ts`
- Indiana binary resolution across standard paths — `main/lib/indiana.ts`
- IPC channel-name registry shared main/preload — `shared/ipc.ts`
- Shared domain types (Note, TreeNode, AnnotationRequest, GitStatusMap) — `shared/domain.ts`

## Known gaps (not yet wired — listed for honesty, not as features)
- Inline Excalidraw canvas — `renderer/src/editor/plugins/ExcalidrawPlugin.tsx` is a dead stub (`NOT WIRED YET`).
- Auto-linking typed URLs; interactive GFM task lists.
- Inline rename, context menu, drag-and-drop move, per-folder create.
- Git blame; external-edit draft reload for an open note.
- `vault:set`, `vault:rel` — main-process handlers exist, but nothing in the renderer UI calls them, so users cannot reach them.
