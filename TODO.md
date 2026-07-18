---
purpose: Track pending work toward GOAL.md. Items are verifiable; not aspirational.
file_rules: rules/template_todo.md
---

# TODO

> Work toward [docs/GOAL.md](docs/GOAL.md). Sequence: [docs/PHASES.md](docs/PHASES.md). Each item names a concrete deliverable and acceptance criteria.


# Do now
- [x] a linter that adds a front-matter to all markdown files — `indiana frontmatter [--write] [path]` (`crates/core/src/frontmatter.rs`)
- [x] a file in .indiana that has the default frontmatter attributes — `.indiana/FRONTMATTER.md`
- [x] a /ship or release command that helps me update the brew package — `/release` (`.claude/commands/release.md` + `.cursor/commands/release.md`)




# LATER: IGNORE
## Hotkey
Copy all from the keyboard, no terminal focus needed.
1. [ ] CLI sets the hotkey.
2. [ ] Menulet helps fix a broken/clashing binding.

## Folder architecture
A new contributor locates any feature from the `crates/` tree and `core/src/` listing without opening a file.
1. [ ] Verify no module crosses boundaries: CLI never imports `daemon` plumbing, core never imports `clap`, protocol crate never depends on core.
2. [ ] List any boundary violation here and fix it.

## MCP server and instructions
A coding agent configured per setup doc calls `read_payload` and gets a valid JSON payload of all markers in a fixture repo.
1. [ ] Verify `indiana mcp` stdio JSON-RPC server works end to end.
2. [ ] Write `docs/indiana/IN_MCP_SETUP.md` — configure Claude Desktop, Cody, Continue to connect to `indiana mcp`.
3. [ ] Document tools exposed: `list_pending_indianas`, `read_indiana { id }`, `read_payload`, `marker_grammar`.
4. [ ] Document daemon-backed fallback: `mcp` does a local scan when no daemon is running.

## Chief of Staff tracker
The task tracker + action log ([COS_PRD.md](docs/chief-of-staff/COS_PRD.md)); the SQLite `todos.db` stub and `indiana todo` are retired.
1. [x] `indiana task add|list|done` + `indiana log` with `--json` for agents.
2. [x] Marker capture: `::todo`/`::task` → Agent queue, `::action` → Human queue, origin backlinks, reconcile on rescan.
3. [x] Daemon appends dispatch lifecycle (`claimed`/`done`/`failed`) to the repo's log.
4. [ ] Decide whether the daemon or MCP exposes tasks, or the CLI stays the only face.
