# Plan — Project switcher + per-project top-bar color (Casablanca)

## Context
Casablanca opens exactly one "vault" (folder) at a time. The single root lives in
`casablanca.config.json` as `{ vaultRootPath }`; switching (via `vault:choose`) overwrites
it, so there is no list of projects and no fast way to jump between them. There is also no
visual cue for *which* repo you're in — the folder pane just says "Files".

Niklas wants to run several projects at once and switch between them, with each project
carrying its own color so it's obvious at a glance which one is active.

Decisions confirmed with the user:
- **Theme = a top-bar color** per project (ambient identity cue). Global light/dark stays as-is.
- **Storage = Casablanca app config** (global, keyed by folder path). Nothing is written into the user's repos.
- **Switcher = a dropdown in the folder-pane header** (sidebar), showing the active project + a matching color dot.

Non-goals: full per-project palettes, per-project light/dark, committing color into repos.

## Design overview
1. Grow the persisted config from one path into a **project list** + an **active path**, each project carrying a `color`. Migrate the legacy `vaultRootPath` on first read.
2. Colors are stored as **space-separated RGB triples** (matching every other token in `styles.css`) and **auto-assigned** from a curated palette on add (first unused color), editable later via a swatch row.
3. Add a `projects:*` IPC surface (list / add / switch / set-color) alongside the existing vault channels; `vault:get` gains the active project's `color`.
4. On switch, the main process re-points the chokidar watcher to the new root (fixes the latent watcher bug) and pushes a fresh tree + git status.
5. Renderer: a slim **TopBar** tinted with the active project's color, a **ProjectSwitcher** dropdown folded into the folder-pane header, and a `--project-color` CSS var driving both.

## Changes

### Pure logic (new, fully unit-tested) — `src/shared/projects.ts`
Keep everything electron/fs-free here so it's testable (config.ts can't be unit-tested directly — it imports `app` from electron at module load). Mirrors the existing pure-helper convention (`flatten-tree.ts`, `resolve-link.ts`).
- `PROJECT_PALETTE: string[]` — ~8 RGB triples chosen to read well under near-white text on the bar in both light/dark.
- `pickColor(used: string[]): string` — first palette entry not in `used`, wrapping by count when all are taken.
- `normalizeConfig(raw): PersistedConfig` — migrate legacy `{ vaultRootPath }` → `{ projects:[{rootPath,color:pickColor([])}], activePath:vaultRootPath }`; dedupe; drop the legacy key.
- `addProjectToConfig(cfg, rootPath)` / `switchActive(cfg, rootPath)` / `setColor(cfg, rootPath, color)` — pure transforms returning a new config; add dedupes by path and sets active.
- `projectName(rootPath)` — basename projection for display.
New `src/shared/projects.test.ts`: migration, dedupe, color distinctness/wrap, add-sets-active, switch, set-color.

### Config persistence — `src/main/lib/config.ts`
- Replace `PersistedConfig` with `{ projects?: ProjectRecord[]; activePath?: string; vaultRootPath?: string }` (last kept only for migration).
- `readPersisted()` runs the raw JSON through `normalizeConfig` so every caller sees the new shape.
- New fns composing the pure helpers with fs: `listProjects()`, `getActiveProject()`, `addProject(rootPath)`, `setActiveProject(rootPath)`, `setProjectColor(rootPath,color)`, `removeProject(rootPath)`.
- Keep `getVaultConfig()`/`setVaultConfig()` as thin shims over the active project so unrelated callers stay working.

### Shared types — `src/shared/domain.ts`
- Add `interface Project { rootPath: string; name: string; color: string; active: boolean }`.
- Extend the ready state: `{ status: 'ready'; rootPath: string; color: string }`.

### IPC — `src/shared/ipc.ts`, `src/main/ipc.ts`, `src/preload/index.ts`
- New channels: `PROJECTS_LIST`, `PROJECTS_ADD`, `PROJECTS_SWITCH`, `PROJECTS_SET_COLOR`, `PROJECTS_REMOVE`.
- `VAULT_GET` returns the active project's `{ status:'ready', rootPath, color }`.
- `PROJECTS_ADD` = the current `VAULT_CHOOSE` dialog, but appends to the list + sets active; `PROJECTS_SWITCH` replaces `VAULT_SET`'s job with validation that the path is known. Retire `VAULT_CHOOSE`/`VAULT_SET` (only `EmptyState`/`useVault` call them).
- Switch/add/remove handlers must: update config, update the `vault` closure var (so `requireVault()`/`getVault()` — used by `registerVaultProtocol` — stay correct), **re-target the watcher**, then `refresh()`.
- Preload: add a `projects` namespace mirroring the channels; keep `vault.get`.

### Watcher re-targeting — `src/main/index.ts`, `src/main/ipc.ts`
- `index.ts` owns `activeWatcher`. Pass a `retargetWatcher(vault: VaultConfig | null)` callback into `registerIpc(sender, { retargetWatcher })` that closes the old watcher and starts a new one (or just closes when null).
- Call it from the switch/add handlers. This fixes the existing bug where switching never moved the watcher off the first root.

### Renderer state — `src/renderer/src/storage/useVault.ts`
- Load `projects` via `window.api.projects.list()` on mount and after any change.
- Add `switchProject(rootPath)`, `addProject()`, `setProjectColor(rootPath,color)`: call IPC, set `vaultState` from the returned ready-state, refresh the project list, and on a *switch* reset `activeNote`/`draft` and the nav stack (paths are per-vault). Existing tree/git effects already re-run because they key off `vaultState`.
- `chooseVault` → delegates to `addProject`.

### Top-bar color — `src/renderer/src/app/theme.ts` (or a small `project-color.ts`), `styles.css`, `tailwind.config.ts`
- Add a `--project-color` var with a neutral default in `:root` / `:root.dark`; map `project: 'rgb(var(--project-color) / <alpha-value>)'` in tailwind config.
- A `useEffect` in `Shell`/`App` sets `--project-color` from `vaultState.color` on change.

### UI components
- **`src/renderer/src/app/TopBar.tsx`** (new): a slim full-width `bg-project` bar (~`h-8`) showing the active project name in a fixed light foreground. Rendered above the pane row.
- **`src/renderer/src/app/Shell.tsx`**: wrap contents in a column — `<TopBar/>` then the existing `folder pane | editor` row. Early-return `EmptyState` when there's no active project (no colored bar with no project).
- **`src/renderer/src/folder-pane/ProjectSwitcher.tsx`** (new): the folder-pane header dropdown — `● {activeName} ▼` (dot = `bg-project`); menu lists every project (color dot + name + ✓ on active), a divider, `＋ Open folder…`, and a compact swatch row to recolor the active project. Selecting a project switches; the folder button (＋ new note) stays.
- **`src/renderer/src/folder-pane/FolderPane.tsx`**: replace the static "Files" label with `<ProjectSwitcher/>`; keep the new-note `+` button. `FileTree` is already keyed by `vaultState.rootPath`, so collapse-state and remount-on-switch already work.
- **`src/renderer/src/app/EmptyState.tsx`**: button label/handler → "Open a project" via `addProject`.

## Verification
- `cd crates/casablanca`
- **Unit**: `npm run test` (vitest) — new `projects.test.ts` covers migration, dedupe, color assignment, switch. Confirm existing suites still pass.
- **Types/build**: `npm run typecheck` (or `build`) clean, including the widened `VaultState` and new preload surface.
- **Manual (dev app)**: `npm run dev`, then:
  1. First launch with an existing `casablanca.config.json` → the old single vault appears as one project with a color (migration).
  2. Open the switcher → "Open folder…" → add a second repo; it gets a *distinct* color and becomes active; the top bar changes color.
  3. Switch back and forth → tree, git tint, top-bar color, and the sidebar dot all track the active project; open note/history reset per project.
  4. Edit a file *externally* in the newly-switched repo → tree refreshes (watcher followed the switch).
  5. Recolor a project via the swatch row → top bar + dot update and the choice survives an app restart.

## Notes / caveats
- The top bar is a personal, per-machine cue (app config) — colors don't travel with the repo, by design.
- Palette colors are pre-vetted for contrast with light text; recoloring is limited to the palette (no free-form picker in v1) to keep every bar readable.
- `--accent` (links, focus) stays blue and independent of project color, per "more a top-bar color."
