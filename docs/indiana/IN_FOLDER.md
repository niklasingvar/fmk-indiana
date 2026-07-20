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
  and any missing `context-model/` / `chief-of-staff/` meta files.
- Existing files are left byte-identical.
- To take a newer embedded default for one command, delete that command's `prompt.md`, then refresh.
- Menulet "update indiana commands" per-folder action delegates to this command via the sidecar.

## Replace
- `indiana templates replace <path>` rewrites every
  `.indiana/indianas/<command>/prompt.md` with the embedded default.
- It is destructive: user edits to command templates are discarded. Use it to
  reset a folder's commands back to the embedded wording.
- Scoped to `indianas/` command templates only — `context-model/` and
  `chief-of-staff/` are not touched.
- Menulet "replace indiana commands" per-folder action delegates to this command via the sidecar.

## Layout

When `indiana add <path>` or `indiana serve <path>` initialises a root:

```
<root>/.indiana/
  SYSTEM_PROMPT.md
  agents/
    mike/SYSTEM_PROMPT.md
    lisa/SYSTEM_PROMPT.md
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
    CONTEXT-MODEL.md
    index.md
    log.md
    purpose/PURPOSE.md
    learnings/INBOX.md
  chief-of-staff/
    README.md
    tasks.md
    log.md
    notes.md
    focus.md
    runs/            (created on first agent turn, not scaffolded)
  casablanca/
    settings.json
```
- `.indiana/SYSTEM_PROMPT.md` — versioned system prompt prepended to every
  agent-facing payload (`indiana copy`, daemon copy, auto-run / group dispatch).
  Authoring source: `crates/core/templates/system_prompt.md`. Scaffolded on
  add/refresh; existing files are left byte-identical. Valid instance wins over
  embedded; invalid falls back with a warning. If the instance `version` is
  behind the embedded version, a warning tells the operator to delete the file
  and refresh — never auto-overwrite. `templates replace` does not touch it.
- `.indiana/agents/<name>/SYSTEM_PROMPT.md` — one file per agent persona, the
  same frontmatter format as `SYSTEM_PROMPT.md` and a full standalone
  replacement for it. A marker tagged `-<name>` (or the unique first letter,
  [IN_COMMANDS.md](IN_COMMANDS.md)) copies and dispatches with that agent's
  prompt. The directory roster *is* the agent registry: adding a folder with a
  valid flag-safe name (lowercase letter first; letters, digits, hyphens)
  defines a new agent, no other wiring. Mike (chief of staff, organizer) and
  Lisa (CTO — systems thinker, domain modeller, architecture) are scaffolded by
  default from `crates/core/templates/agents/`; existing files are never
  overwritten.
- Command templates live under `.indiana/indianas/`, one folder per long marker name:
  - `.indiana/indianas/fix/prompt.md`
  - `.indiana/indianas/question/prompt.md`
  - `.indiana/indianas/hate/prompt.md`
  - same for `love`, `keep`, `elaborate`, `note`, `action`, `todo`, `delete`, `prompt`
- `.indiana/context-model/` — per-repo memory ([CM_PRD.md](../context-model/CM_PRD.md)).
  Scaffolded with seed files: the schema (`CONTEXT-MODEL.md`), the journal
  (`index.md`, `log.md`), `purpose/PURPOSE.md`, and `learnings/INBOX.md`.
  The tree grows from there per the schema's own rules.
- `.indiana/chief-of-staff/` — focus management: `README.md`, `notes.md`,
  `focus.md` (one-line seeds) plus the task tracker `tasks.md` and action log
  `log.md` ([COS_PRD.md](../chief-of-staff/COS_PRD.md)). tasks.md and log.md are
  scaffolded as skeletons and also self-seed on first machine write, so capture
  works in roots scaffolded before they existed. `indiana task add|list|done`
  and `indiana log` are the CLI face; `--json` is the agent surface. (A stale
  `todos.db` from the retired `indiana todo` stub is inert; delete freely.)
  `runs/` holds one machine-written audit record per agent turn — facts as
  YAML frontmatter, transcript as body, faced by `indiana runs --json`
  ([COS_PRD.md](../chief-of-staff/COS_PRD.md)); created on first dispatch,
  pruned to the newest 200, safe to delete, history only.
- `.indiana/casablanca/settings.json` — per-repo [Casablanca](../casablanca/CASABLANCA_OVERVIEW.md)
  settings: a committable JSON bag the editor, the daemon, and the CLI share.
  Created on first write, not by scaffolding. Known keys: `color` (the editor's
  project color, overrides its global registry), `theme` (`light` | `dark` —
  Casablanca editor palette; no in-app toggle), `autoRun` (per-repo auto-run
  opt-in the daemon reads), `model` (optional ACP model value selected before
  the turn, [IN_AUTORUN.md](IN_AUTORUN.md)), and `maxRowsPerFile` (the elephant
  ceiling, [FUNDAMENTALS.md](../../FUNDAMENTALS.md); declared, no reader yet);
  unknown keys are ignored. `indiana
  casablanca get|set|settings|path` is the CLI face — a per-repo input store, not
  derived from source and not part of the marker index.

## Source of truth

`init_folder_indiana` scaffolds every monitored root from embedded defaults so
the on-disk `.indiana/` is real, editable content — not derived cache:
- Everything comes from `crates/core/templates/`, the single authoring source:
  command templates (`indianas/<command>/prompt.md`, full files written
  verbatim), the versioned system prompt (`system_prompt.md` →
  `.indiana/SYSTEM_PROMPT.md`), and meta folder seeds (`context-model/`,
  `chief-of-staff/`). Edit a file there to change what new monitored roots
  start with.
- A root's existing file always wins; delete it and `indiana templates refresh`
  to re-seed. Existing files on disk are left byte-identical.
- The `test_embedded_templates_match_marker_table` test fails if a template's
  frontmatter drifts from its marker row.

In the Indiana repository itself, `.indiana/` is a dogfood instance like any
other monitored repo — user input that tunes wording for existing marker
kinds, free to diverge from the embedded defaults.

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
- Embedded `crates/core/templates/` wins when no valid folder template exists.
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
- The embedded default is authored in `crates/core/src/frontmatter.rs`
  (`DEFAULT_FRONTMATTER`). This repo's `.indiana/FRONTMATTER.md` is dogfood
  instance config like any other root's. The file is not scaffolded into
  monitored roots — those use the embedded default unless a user adds it.
