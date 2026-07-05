---
status: draft
purpose: High-level phased plan — get the whole system running daily, editor-first. Detail stays in the specs.
approval: pending
---

# ACTION_PLAN — phases

> Concern map: [MENTAL_MODEL.md](MENTAL_MODEL.md). System shape: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Supersedes docs/PHASES.md.
> North star: open a repo in Casablanca, edit markdown inline, tag `::` markers, hit `Copy all`, paste into the agent, watch the fix land — zero terminal.

## Phase 0 — Truth alignment (docs only)
- [x] Canonical names adopted ([MENTAL_MODEL.md](MENTAL_MODEL.md)); COS, Boxydoc, Bangalore retired from paths and specs.
- [x] Doc folders renamed: `docs/indiana/`, `docs/menulet/`, `docs/casablanca/`, `docs/context-model/`, `docs/montmartre/`.
- [x] Dead relative links fixed; CLAUDE.md updated.
- [x] VISION.md aligned with reality: `.indiana/indianas/` layout, context-model naming, custom command kinds marked as planned, wedge/destination section added.
- [x] Montmartre declared vision-first, architecture later ([MM_VISION.md](docs/montmartre/MM_VISION.md)).
- [x] `temp_system.md` deleted; `COS_PRD.md` rewritten as [CM_PRD.md](docs/context-model/CM_PRD.md).
- [x] Templates given one home: `crates/core/templates/` (full `prompt.md` files absorb `prompts.toml` + the Rust scaffold codegen + `scaffold/`). This repo's `.indiana/` is dogfood, free to diverge; `test_embedded_templates_match_marker_table` replaces the repo-mirror test.
- [x] `.indiana/indianas/RULES.md` moved to [IN_COMMAND_RULES.md](docs/indiana/IN_COMMAND_RULES.md) — spec content, not instance data.
- [x] Editor identity settled (2026-07): Casablanca is the editor, self-built at `crates/casablanca/`; nimbalyst is vendored reference only; presentations are a feature ([CASABLANCA_PRD.md](docs/casablanca/CASABLANCA_PRD.md)).
- [ ] Sweep remaining docs (IN_*, MENULET_*) for stale claims. (Frontmatter part done: `indiana frontmatter` now flags only the four montmartre template seeds — that call is Phase 5's.)
- Exit: `rg -i 'bangalore|boxydoc|\bCOS\b|visualviewer|\bnimbus\b'` finds nothing outside retired-alias notes and git history; every doc link resolves.

## Phase 1 — Casablanca MVP: the daily loop
The goal state. Prototype exists (3-pane shell, vault, Lexical WYSIWYG, autosave, watcher); what's missing is safety and the button. Detail: [crates/casablanca/TASKS.md](crates/casablanca/TASKS.md).
- Round-trip safety first — without it the editor corrupts the repo it reviews:
  - `NoteDocument` domain split; YAML frontmatter preserved as an opaque block (TASKS track 0).
  - `::` marker lines survive open → autosave byte-stable; fixture tests with every marker kind.
- `Copy all` button: main process shells `indiana copy` with cwd = vault root; toast on success/failure; resolve the binary like the menulet does (standard locations, not shell PATH); friendly hint when `indiana` is missing.
- Rich-editing basics: tables + links (TASKS track 2), frontmatter panel (track 1).
- Exit: open fmk-indiana in Casablanca, edit a doc inline, tag `::fix`, press `Copy all`, paste into Claude Code, the fix lands, the editor refreshes. Zero terminal, zero corrupted files.

## Phase 2 — One system, running daily
Glue so all faces coexist on one machine, every day.
- Daemon under `launchd` (`indiana service install`); menulet, CLI, and Casablanca all talk to the same daemon.
- Opening a vault that isn't monitored offers `indiana add` — vault and monitored root become the same folder.
- Autosave/watcher coexistence verified: editor writes don't churn the daemon (own-write tracking), daemon-side agent edits refresh the editor without losing caret/collapse state.
- Menulet counts match what the editor shows.
- Exit: a full week of real work where the loop never needs a terminal and nothing fights.

## Phase 3 — Close the agent loop over MCP
- Verify `indiana mcp` end to end from a real harness (TODO.md): agent calls `read_payload`, gets valid JSON for a fixture repo.
- Write `docs/indiana/IN_MCP_SETUP.md` for Claude Code / Cursor.
- Exit: one full loop — tag, agent pulls over MCP, agent edits, marker resolved — zero clipboard. (`Copy all` stays as the universal fallback.)

## Phase 4 — Distribution
- Package Casablanca: electron-builder DMG (unsigned first, same as menulet's path).
- Cask in the tap (`niklasingvar/homebrew-fmk-indiana`) beside `indiana-menulet`; extend `/release` to build CLI + menulet + Casablanca.
- Exit: `brew install --cask indiana-casablanca` on a friend's Mac; the diagram's "brew → three products" is literally true.

## Phase 5 — Context-model becomes real
- [x] Define the read/write contract: schema authored in `files/CONTEXT-MODEL.md`, shipped as the seed `CONTEXT-MODEL.md`; [CM_PRD.md](docs/context-model/CM_PRD.md) points at it.
- [x] Ship seed files in `crates/core/templates/context-model/` (schema, index, log, purpose, learnings INBOX) instead of a bare `.gitkeep`.
- [ ] Decide seed frontmatter: the montmartre seeds ship without it and the linter flags them; adding it changes shipped bytes (and the byte-exact scaffold test). (Context-model seeds ship with schema frontmatter.)
- [x] Wire the templates: `render_text` prepends a loop preamble (read protocol + log/focus.md write-back); `::hate`/`::love`/`::note` instruct the INBOX write-back; `::todo` lands in `.indiana/montmartre/focus.md`.
- Exit: give feedback once in a fixture repo; the next compiled payload carries the learned rule.

## Phase 6 — Presentations as a feature
- Inline Excalidraw (DecoratorNode + fenced-block transformer — TASKS phase 4).
- Rendered deck view from template-first, content/design-separated files ([VISION.md](VISION.md) presentation flow); annotation boxes emit ordinary `::` markers into source.
- Exit: review one real deck end to end without opening a code editor.

## Phase 7 — Montmartre vision, then design
- Settle [MM_VISION.md](docs/montmartre/MM_VISION.md): what a ticket is, queue semantics, what "focus" means operationally.
- Only then write MM_PRD and build the human/agent queues; menulet gains the focus view after that.
- Exit: "what should I be doing right now?" answered by one glance at the menulet.

## Phase 8 — Auto-run
- Daemon dispatches compiled markers to the agent as they appear; pausable for batching.
- Requires Phase 3 proven and the write-chokepoint guarantees extended to dispatch.
- Exit: mark `::fix`, keep reading, the fix lands without a run command.

## Parked toward VISION (planned, not scheduled)
- Per-repo custom command kinds. Today grammar is global and only wording is tunable ([IN_FOLDER.md](docs/indiana/IN_FOLDER.md)); VISION wants user-defined commands (`::design`). Needs a marker-table extension mechanism that keeps the one-table invariant.
- Reconcile [IN_COMMAND_RULES.md](docs/indiana/IN_COMMAND_RULES.md) with the marker TABLE: the rules list three valid `command_type` values, but templates use `reaction`, `user_context`, `user_task`, `agent_explains`, `agent_run_directly` too. Spec wins or spec changes.
- `question_empty` variant (`crates/core/templates/indianas/question/prompt_empty.md`) is embedded-only — not scaffolded into instances and not overridable per root. Decide whether variants should be.
- Human-edit version handling on rendered views.
- Global/cross-project context-model.
- `::todo` markers vs `todos.db`: import, sync, or unrelated.

## Ordering rationale
- 0 before all: every later phase writes docs; writing onto a misaligned base compounds drift.
- 1 before everything else that builds: the editor loop is the product being dogfooded; running it daily generates the feedback that steers 2–8.
- 1 before 2: no point wiring a system around an editor that corrupts files.
- 3 before 8: auto-run rides the proven MCP loop.
- 5 before 8: auto-run without memory repeats mistakes faster.
- 4 floats: ship whenever the loop is worth sharing.
