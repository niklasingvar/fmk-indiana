---
status: draft
purpose: Specify repo-local `.indiana` command templates.
approval: pending
---

# IN_FOLDER — folder-local templates

> Marker grammar: [IN_COMMANDS.md](IN_COMMANDS.md). Compile path: [IN_PRINCIPLES.md](IN_PRINCIPLES.md). Daemon ownership: [IN_DAEMON.md](IN_DAEMON.md).

## Intent
- `.indiana/` is repo-local user input.
- It tunes compiled prompt wording for existing marker kinds.
- It does not add marker kinds. Grammar stays global in `crates/core/src/markers.rs`.
- It is not derived cache. `indiana remove` must not delete it.
- User customization means editing `prompt.md` for an existing command, not inventing a new `::` token.

## Creation
- `indiana add <path>` creates `<path>/.indiana/` when missing.
- `indiana serve <path>` also initializes it for that monitored root.
- Existing files are left byte-identical.
- Menulet add uses daemon add, so it gets the same initialization.

## Refresh
- `indiana templates refresh <path>` creates any missing command template files
  and any missing `context-model/` / `montmartre/` meta files.
- Existing files are left byte-identical.
- To take a newer embedded default for one command, delete that command's `prompt.md`, then refresh.
- Menulet "update indiana commands" per-folder action delegates to this command via the sidecar.

## Replace
- `indiana templates replace <path>` rewrites every
  `.indiana/indianas/<command>/prompt.md` with the embedded default.
- It is destructive: user edits to command templates are discarded. Use it to
  reset a folder's commands back to the embedded wording.
- Scoped to `indianas/` command templates only — `context-model/` and
  `montmartre/` are not touched.
- Menulet "replace indiana commands" per-folder action delegates to this command via the sidecar.

## Layout

When `indiana add <path>` or `indiana serve <path>` initialises a root:

```
<root>/.indiana/
  indianas/
    fix/prompt.md
    question/prompt.md
    hate/prompt.md
    love/prompt.md
    keep/prompt.md
    elaborate/prompt.md
    note/prompt.md
    action/prompt.md
    todo/prompt.md
    delete/prompt.md
    prompt/prompt.md
  context-model/
    .gitkeep
  montmartre/
    README.md
    actions.md
    notes.md
    focus.md
    todos.db
```
- Command templates live under `.indiana/indianas/`, one folder per long marker name:
  - `.indiana/indianas/fix/prompt.md`
  - `.indiana/indianas/question/prompt.md`
  - `.indiana/indianas/hate/prompt.md`
  - same for `love`, `keep`, `elaborate`, `note`, `action`, `todo`, `delete`, `prompt`
- `.indiana/context-model/` — current state, direction, rules. Scaffolded empty
  (a `.gitkeep` only); the user fills it in per repo.
- `.indiana/montmartre/` — project management: `README.md`, `actions.md`,
  `notes.md`, `focus.md`, each seeded with a one-line header.
- `.indiana/montmartre/todos.db` — the Montmartre todo list (SQLite). Created
  on first `indiana todo` command, not by `add`/`refresh` scaffolding. See
  [IN_PRINCIPLES.md](IN_PRINCIPLES.md) Montmartre carve-out: it is authoritative
  state separate from `::todo` markers. Fields: `id` (Indiana-style), `todo`
  (max 29 words), `domain`, and `dependencies` (ids of existing todos). `indiana
  todo add|list|delete` is the read/write face; `--json` is the agent surface.

## Source of truth

`init_folder_indiana` scaffolds every monitored root from embedded defaults so
the on-disk `.indiana/` is real, editable content — not derived cache:
- Command prompt bodies: `crates/core/prompts.toml`. Edit a word there to change
  what a fresh `prompt.md` starts with. A root's existing `prompt.md` wins over
  this; delete the file and `indiana templates refresh` to re-seed.
- Meta folder seeds (`context-model/`, `montmartre/`): `crates/core/scaffold/`.
  Edit those files to change what new monitored roots start with. Existing files
  on disk are left byte-identical; delete one and re-`add`/`refresh` to re-seed.

In the Indiana repository itself, `.indiana/indianas/<command>/prompt.md` is the
authoring source for the embedded defaults: edit the repo template first, then
mirror the wording into `crates/core/prompts.toml` and the marker row. The
`test_repo_indianas_match_embedded_defaults` test fails if they drift. In every
other monitored repo, `.indiana/` remains user input that tunes wording for
existing marker kinds.

## Frontmatter contract
- Each `prompt.md` starts with YAML frontmatter:

```yaml
---
status: draft
purpose: Folder-local prompt template and behavior for ::fix.
approval: pending
command: fix
command_type: agent_directive
message: optional
---
```

- `command` must match the folder name.
- Missing or invalid frontmatter makes that file fall back to embedded defaults.
- The first non-heading paragraph is the compiled prompt template.
- `{message}` is replaced with the marker message when present.
- Scaffolded files open with a `# ::<command> — …` heading line for human
  readability. It is **not** part of the compiled prompt — the compiler skips
  `#` lines, so only the paragraph below it is used. Edit or delete the heading
  freely; it never reaches `indiana copy` or MCP output. For passthrough kinds
  (`note`, `action`, `todo`) the compiled prompt is just the user's message, so
  the body is `{message}` by design.

## Precedence
- Folder template wins for markers under that monitored root.
- Embedded `crates/core/prompts.toml` wins when no valid folder template exists.
- Multiple monitored roots use longest path ownership.

## Scan exclusion
- `.indiana/` is excluded from normal markdown scanning.
- Command templates never become review content.

## Default frontmatter linter
- `indiana frontmatter [path]` reports `.md` files under `path` (default `.`)
  that lack frontmatter; `--write` prepends the default block.
- The default block is read from `<root>/.indiana/FRONTMATTER.md` — the
  authoring source — and falls back to an embedded constant
  (`crates/core/src/frontmatter.rs`) when that file is absent or unparseable.
- The walk respects `.gitignore` and prunes `.indiana/`, `.git/`, `target/`,
  `node_modules/`, and `skills/` (third-party skill content keeps its own
  frontmatter convention). Files that already open with a `---` fence are left
  alone; the linter never rewrites existing frontmatter.
- In the Indiana repository, `.indiana/FRONTMATTER.md` is the authoring source
  for the embedded default; edit the block there and mirror it into
  `DEFAULT_FRONTMATTER`. It is not scaffolded into other monitored roots —
  those use the embedded default unless a user adds the file.
