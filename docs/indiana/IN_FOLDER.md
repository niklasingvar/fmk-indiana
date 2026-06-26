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
- `indiana templates refresh <path>` creates any missing command template files.
- Existing files are left byte-identical.
- To take a newer embedded default for one command, delete that command's `prompt.md`, then refresh.
- Menulet "update indiana commands" per-folder action delegates to this command via the sidecar.

## Layout

When `indiana add <path>` or `indiana serve <path>` initialises a root:

```
<root>/.indiana/
  fix/prompt.md
  question/prompt.md
  hate/prompt.md
  love/prompt.md
  keep/prompt.md
  elaborate/prompt.md
  note/prompt.md
  action/prompt.md
  todo/prompt.md
```
- One folder per long marker name:
  - `.indiana/fix/prompt.md`
  - `.indiana/question/prompt.md`
  - `.indiana/hate/prompt.md`
  - same for `love`, `keep`, `elaborate`, `note`, `action`, `todo`

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

## Precedence
- Folder template wins for markers under that monitored root.
- Embedded `crates/core/prompts.toml` wins when no valid folder template exists.
- Multiple monitored roots use longest path ownership.

## Scan exclusion
- `.indiana/` is excluded from normal markdown scanning.
- Command templates never become review content.
