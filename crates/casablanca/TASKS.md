# Casablanca — Task List

Three feature tracks, each grounded in the domain model. The guiding principle:
**parsing/serialization concerns live in the shared domain layer, filesystem
concerns live in main, and the renderer only composes application hooks +
presentation components.**

## 0. Domain foundation (do first — the other tracks depend on it)

- [ ] Introduce a `NoteDocument` value object in `src/shared/domain.ts` that splits a raw
      markdown file into `{ frontmatter: string | null, body: string }`
  - [ ] Add a pure `note-serialization` module in `src/shared/` with `parseNoteDocument(raw)`
        and `serializeNoteDocument(doc)` — frontmatter is treated as an opaque YAML text block
        (delimited by `---` fences at the top of the file); round-trip must be byte-stable
  - [ ] Unit-test the round-trip: no frontmatter, empty frontmatter, frontmatter with `---`
        inside strings, body starting with a thematic break
- [ ] Refactor `useVault` so the draft state is a `NoteDocument`, not a raw string
  - [ ] Parse once when a note is opened; serialize once on autosave — the Lexical editor
        only ever sees the `body`
- [ ] Decide and document the module layout in `app/README.md` (bounded contexts:
      **vault** = filesystem projection in main, **note authoring** = editor in renderer,
      **shell** = layout/orchestration)

## 1. Frontmatter surfaced as a code snippet at the bottom

- [ ] Render a `FrontmatterPanel` component below the editor in `EditorPane`
  - [ ] Display the raw YAML in a monospace code block styled like the editor's code blocks
  - [ ] Hide the panel entirely when the note has no frontmatter
  - [ ] Label it clearly (e.g. a small "Properties" / "frontmatter" header) so it reads as
        metadata, not document content
- [ ] Make the snippet editable as plain text (textarea styled as code), writing back into
      the draft `NoteDocument` so autosave persists it
  - [ ] Validate only the fence structure, not YAML semantics — the domain treats
        frontmatter as opaque
- [ ] Verify externally-edited frontmatter (file watcher) round-trips without loss

## 2. Formatted markdown: tables + link handling

### Tables
- [ ] Add `@lexical/table` and register `TableNode`, `TableRowNode`, `TableCellNode`
      in the editor config, plus `TablePlugin`
- [ ] Add a markdown table transformer to `MARKDOWN_TRANSFORMERS`
      (port the Lexical playground's `TABLE` element transformer if the installed
      version doesn't export one)
- [ ] Theme tables (borders, header row, cell padding) consistent with the existing
      editor theme in `Editor.tsx` / `styles.css`
- [ ] Table UX: tab/shift-tab cell navigation, a minimal row/column insert-delete
      affordance (context menu or hover handles)
- [ ] Round-trip test: GFM table markdown → editor → markdown is stable

### Links
- [ ] Register `AutoLinkNode` + `AutoLinkPlugin` with URL/email matchers so typed
      URLs become links
- [ ] Add `ClickableLinkPlugin` so cmd/ctrl+click follows links
- [ ] Route all external navigation through the main process: `setWindowOpenHandler`
      + `will-navigate` → `shell.openExternal`, deny in-app navigation (security boundary —
      belongs in `src/main/index.ts`)
- [ ] Floating link editor: on caret-in-link, show a small popover to view/edit/remove
      the URL (use `@floating-ui/react`; never hand-roll fixed positioning)
- [ ] Decide handling of vault-internal links (`[note](./other.md)`): open in the editor
      via `openNote` instead of the browser — resolve relative to the note's folder in
      the vault domain, not in the UI component

## 3. Nicer file tree (nimbalyst-inspired)

### Visual polish
- [ ] Replace text arrows (`▾`/`▸`) with proper chevron icons and add file/folder icons
      (folder open/closed state, markdown file icon)
- [ ] Indent guides (vertical rulers per depth level), tighter row height, hover +
      active states aligned with the pane color tokens in `tailwind.config.ts`
- [ ] Sort: folders first, then files, case-insensitive natural sort — as a pure function
      in the vault domain (`src/shared/`), not inline in the component
- [ ] Empty-folder and root-level affordances ("New note", "New folder" buttons in the
      pane header)

### Structure & state
- [ ] Split `FileTree` into `FileTreeRow` (pure presentation: icon, name, states) and a
      container that owns tree state — mirrors nimbalyst's `FlatFileTree`/`FileTreeRow` split
- [ ] Lift expanded/collapsed state out of per-node `useState` into a single
      `expandedPaths: Set<string>` owned by the folder-pane container, persisted per vault
      (survives restarts and tree refreshes — today every watcher refresh re-mounts nodes
      and loses collapse state)
- [ ] Keyboard navigation: up/down to move, left/right to collapse/expand, enter to open

### File operations
- [ ] Context menu (rename, delete, new note here, new folder, reveal in Finder) using
      `@floating-ui/react`
- [ ] Inline rename (auto-select name without extension, Enter/Escape), backed by a new
      `notes.rename` IPC handler in the vault domain
- [ ] Delete with confirmation, moving to OS trash (`shell.trashItem`) rather than
      permanent delete
- [ ] Drag-and-drop move between folders (drop-target highlight; vault domain exposes a
      `move(fromRel, toRel)` operation that validates paths stay inside the root)

## Suggested order

1. Track 0 (domain foundation) — small, unblocks frontmatter and keeps the editor honest
2. Track 1 (frontmatter panel) — quick win once the domain split exists
3. Track 2 (tables + links) — self-contained in the editor context
4. Track 3 (file tree) — largest track; visual polish first, then operations
