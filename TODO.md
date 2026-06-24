---
status: draft
purpose: Track pending work toward GOAL.md. Items are verifiable; not aspirational.
approval: pending
document_policy: ::fix
---

# TODO

> Work toward [docs/GOAL.md](docs/GOAL.md). Sequence: [docs/PHASES.md](docs/PHASES.md). Each item names a concrete deliverable and acceptance criteria.

## Copy all, copy latest
- `indiana copy` copies every marker — Phase 3 baseline, done.
- `indiana copy --latest` — deferred. Design exists in [plans/copy_latest_cursor_f82456a5.plan.md](plans/copy_latest_cursor_f82456a5.plan.md). Depends on a cursor store; not yet implemented.

## Copy actions — done
- `indiana copy --kind action` copies only `::action` / `::todo` markers.
- `indiana copy --kind note` copies only `::note`.
- Combines with `--latest`: `indiana copy --kind action --latest` → only new actions (when `--latest` lands).
- Kind filter lives in the compile step (`compile.rs`), not in the CLI face only — MCP `read_payload` gets the same filter. Verified.

## CLI-first principle in CLAUDE.md — done
- `## CLI first` section added to [CLAUDE.md](CLAUDE.md) — rules live there now.
- Help snapshot: `tests/cli_help.snap` + `test_cli_help_snapshot` in CI (`ci.yml`) catches CLI drift. Verified.


## Hotkey First
- Add so we can copy all from the keyboard
- CLI sets the hotkey? Menulet helps fixing it

## .indiana folder
- contains folder markdown skills


## Folder architecture
- Audit `crates/` tree: `indiana` (CLI), `core` (domain), `indiana-protocol` (socket wire format). This is the intended split per [IN_ARCHITECTURE.md](INDIANA/IN_ARCHITECTURE.md).
- `core/src/` sub-modules (`parser.rs`, `compile.rs`, `scope.rs`, `index.rs`, `write.rs`, `markers.rs`, `id.rs`, `walk.rs`) already reflect folder-as-architecture — one file per responsibility.
- Verify no module crosses boundaries: CLI never imports `daemon` plumbing, core never imports `clap`, protocol crate never depends on core.
- If any boundary violation exists, list it here and fix.
- Acceptance: a new contributor can locate any feature by reading the `crates/` tree and `core/src/` file listing without opening a file.


## MCP server and instructions 
- `indiana mcp` starts a stdio JSON-RPC server — implementation exists in `mcp.rs`. Verify it works end to end.
- Write `INDIANA/IN_MCP_SETUP.md` — instructions for configuring coding agents (Claude Desktop, Cody, Continue, etc.) to connect to `indiana mcp`.
- Configured agent gets these tools:
  - `list_pending_indianas` — all markers not yet done/failed.
  - `read_indiana { id }` — one marker by ID.
  - `read_payload` — full compiled payload (all markers).
  - `marker_grammar` — the marker table so the agent knows what each kind means.
- Acceptance: a coding agent configured per IN_MCP_SETUP.md can call `read_payload` and receive a valid JSON payload of all markers in a fixture repo.
- Stretch: daemon-backed MCP (the `mcp` command talks to the daemon socket instead of doing its own scan). Currently falls back to a local scan when no daemon — acceptable for Phase 4, but document the behavior.
