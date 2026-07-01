---
name: create-indiana-command
description: Add a new Indiana `::` marker command end-to-end. Use when the user asks to create, add, or define a new Indiana command or marker (e.g. "::delete", "add a new indiana"), or to wire a drafted `.indiana/indianas/<command>/prompt.md` into the parser, compiler, scaffold, counts, docs, and tests.
---

# Create an Indiana command

## Source of truth

In this repository, `.indiana/indianas/<command>/prompt.md` is the authoring source for default command templates. Embedded defaults (`crates/core/prompts.toml`) and scaffold generation mirror it. The `test_repo_indianas_match_embedded_defaults` test in `crates/core/src/templates.rs` fails if they drift.

## Read first

- `CLAUDE.md`
- `docs/indiana/IN_COMMANDS.md` — grammar and the set
- `docs/indiana/IN_FOLDER.md` — frontmatter contract and layout
- `docs/indiana/IN_PRINCIPLES.md` — one marker table drives everything; content is data

## Gather before editing

Ask the user (one focused question) for anything not given:

- long token (e.g. `delete`) and short token(s) (e.g. `d`)
- message contract: `none` / `optional` / `required`
- tracked? (only `::action` / `::todo` are tracked today)
- `command_type` (see vocabulary below)
- prompt body wording — the compiled prompt template; use `{message}` where the marker message goes

## `command_type` vocabulary

- `agent_directive` — agent acts directly (`::fix`, `::elaborate`)
- `agent_explains` — agent explains to the user (`::question`)
- `agent_gated_directive` — agent prepares the change, then checks in with the user before acting (`::delete`)
- `agent_run_directly` — auto-calls a code agent to act on the prompt (`::prompt`)
- `reaction` — user reaction, no message (`::hate`, `::love`, `::keep`)
- `user_context` — user-authored context, passthrough (`::note`)
- `user_task` — user task, passthrough, tracked (`::action`, `::todo`)

`command_type` is marker metadata declared on the `TABLE` row in `crates/core/src/markers.rs`, not derived from `Kind`.

## Apply the change

1. Author `.indiana/indianas/<command>/prompt.md` in the standard contract: frontmatter `status`, `purpose`, `approval`, `command` (must equal the folder name), `command_type`, `message`; then a `# ::<command> — …` heading; then the prompt body. The first non-heading paragraph is the compiled template.
2. Add the prompt body to `crates/core/prompts.toml`, keyed by the long token.
3. Add a `Kind` variant and a row to `TABLE` in `crates/core/src/markers.rs`. Set `command_type` on the row.
4. Add a `# ::<command> — …` arm to `heading()` in `crates/core/src/templates.rs`.
5. Add the kind to `Counts`, `counts()`, and `Counts::total()` in `crates/core/src/index.rs`, and to the scan-rendering list in `crates/indiana/src/main.rs`.
6. Update `docs/indiana/IN_COMMANDS.md` (the set table, Types, compiled prompt) and `docs/indiana/IN_FOLDER.md` (layout list).
7. Update the table-count test in `markers.rs` and add a compile/scaffold test for the new kind.

## Verify

- `cargo test -p indiana-core` and `cargo test -p indiana` pass.
- `test_repo_indianas_match_embedded_defaults` passes — repo templates equal embedded defaults.
- Local smoke (optional, per the workspace testing rule): `make scratch` and `make serve`.

## Caveats

- `Counts` is still a per-kind struct, not table-driven. Adding a marker means touching `Counts`, `counts()`, `total()`, and the CLI scan list — not only `TABLE`. Flag this drift when you add a marker; do not refactor it unless asked.
- `command_type` is validated as non-empty frontmatter and surfaced in MCP grammar output, but the compiler still uses only the prompt body. `command_type` is a contract for faces and humans, not a runtime branch.
