---
name: CLI Copy Filters
overview: "Implement the TODO block’s actionable work now: `copy --kind`, CLI-first dynamic help, help snapshot enforcement, and doc-link drift fixes. Defer `copy --latest` cursor/database design to the next phase per your instruction."
todos:
  - id: core-kind-filter
    content: Add core compile options and marker-table-backed kind filtering.
    status: pending
  - id: cli-copy-kind
    content: Wire `indiana copy --kind` through core compile options.
    status: pending
  - id: mcp-kind-filter
    content: Expose the same kind filter on MCP `read_payload`.
    status: pending
  - id: help-snapshot
    content: Generate marker help from `TABLE` and add help snapshot enforcement.
    status: pending
  - id: docs-drift
    content: Fix docs links and mark `--latest` deferred.
    status: pending
  - id: verify
    content: Run Rust tests and local Indiana smoke test.
    status: pending
isProject: false
---

# CLI Copy Filters Plan

## Assumptions
- `indiana copy --latest` is deferred. Do not add cursor files, timestamps, or database state in this pass.
- `--kind action` means both `::action` and `::todo`, matching `TODO.md`.
- Menulet gets no new computation. If later UI is added, it passes CLI-equivalent options through to Indiana.

## Implementation
- Add core compile options in [`crates/core/src/compile.rs`](crates/core/src/compile.rs):
  - Keep `compile(&Index)` as the default all-marker entry point.
  - Add `compile_with_options(&Index, &CompileOptions)` with `kind: Option<KindFilter>`.
  - Filter before `compile_marker`, so CLI and MCP share the same behavior.

- Centralize kind names in [`crates/core/src/markers.rs`](crates/core/src/markers.rs):
  - Add a public long-name helper backed by `TABLE`.
  - Add `KindFilter` parsing from marker table tokens.
  - Special-case `action` filter to match `Kind::Action | Kind::Todo`.
  - Remove or stop relying on duplicated CLI kind-name matches where touched.

- Wire CLI in [`crates/indiana/src/main.rs`](crates/indiana/src/main.rs):
  - Add `copy --kind <kind>`.
  - Parse via core marker helpers, not local string translation.
  - Call `compile_with_options`.
  - Report copied count from `payload.markers.len()`, not pre-filtered `idx.markers.len()`.
  - Generate marker help text from `TABLE` using clap `after_help`/equivalent so adding a marker row changes help without editing command prose.

- Wire MCP parity in [`crates/indiana/src/mcp.rs`](crates/indiana/src/mcp.rs):
  - Add optional `kind` argument to `read_payload` input schema.
  - Compile from the current index with the same `CompileOptions`.
  - Do not filter rendered JSON after compilation.

- Add help drift enforcement:
  - Add [`crates/indiana/tests/cli_help.snap`](crates/indiana/tests/cli_help.snap).
  - Add `test_cli_help_snapshot` in [`crates/indiana/tests/cli.rs`](crates/indiana/tests/cli.rs) that runs `indiana help` and compares normalized stdout to the snapshot.
  - Add minimal CI if none exists: [`.github/workflows/ci.yml`](.github/workflows/ci.yml) running `cargo test --release`. The snapshot test is the drift check.

- Fix docs drift:
  - Update [`CLAUDE.md`](CLAUDE.md) read-first links from root docs to [`docs/PURPOSE.md`](docs/PURPOSE.md), [`docs/GOAL.md`](docs/GOAL.md), [`docs/PHASES.md`](docs/PHASES.md).
  - Update [`TODO.md`](TODO.md) top links the same way.
  - Update the TODO text to say `--latest` is deferred to database/cursor phase.

## Verification
- Add core tests for `compile_with_options`:
  - `kind=note` returns only notes.
  - `kind=action` returns action and todo, not hate/love/fix.
  - Unknown kind fails before compile at the face boundary.

- Add CLI tests:
  - Fixture with `::hate`, `::action`, `::todo`, `::note`; `indiana copy --kind action` returns exactly action + todo.
  - `indiana copy --kind note` returns exactly one note.
  - `indiana help` snapshot includes marker kinds generated from `TABLE`.

- Add MCP test:
  - `read_payload { "kind": "action" }` returns action + todo only.

- Run:
  - `cargo test --release`
  - `make scratch`, `make serve`, `make add`, `make scan`, `make copy` for local smoke after tests pass.