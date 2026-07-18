# Chief-of-staff task tracker + action log, Casablanca viewer, CLI, cone-tree skill

## Context

The auto-run loop works end to end, but nothing durable records what happened: agent
jobs and transcripts are daemon memory that die with the turn (only trace: raw text at
`~/.indiana/dispatch/<id>.log`, outside the repo). Task state is split across a
placeholder SQLite `todos.db`, `focus.md` checkboxes, and `::todo`/`::action` markers —
the parked "`::todo` vs `todos.db`" question from ACTION_PLAN.md.

This plan makes chief-of-staff start to bake (Phase 7): one repo-local, human-editable,
token-efficient store under `.indiana/chief-of-staff/` — a **task tracker** (two queues
per COS_VISION) and an **action log** (append-only machine record of what ran). Plus a
Casablanca panel to view both and natively add markers, CLI read/write, the vision/PRD
docs, and a **cone-shaped-tree architecture skill** (added mid-planning) so agents
uphold the domain-tree structure named in FUNDAMENTALS.md.

## Decided with the user

- One subsystem: log + tracker. Owner: chief-of-staff (`docs/chief-of-staff/`,
  `.indiana/chief-of-staff/`).
- Storage: hand-editable, token-efficient markdown line formats — not JSONL/SQLite/JSON.
- Queues: `::todo` and `::task` → Agent queue; `::action` → Human queue. A captured
  marker gets an ID and a tracker line with an origin backlink.
- `todos.db` + `indiana todo` are **replaced** (rusqlite dependency drops out).
- Act-on-task v1: click a task → jump to origin file/line (source stays the single
  truth). Dispatch-from-panel deferred.
- `::action` compiled prompt retargeted: human-queue item, agent must not execute it.

## Verified ground truth (build on, don't rebuild)

- `crates/core/src/walk.rs` `PRUNE = [".indiana", ".git"]` — the scan excludes
  `.indiana/`, so tracker-internal markers are inert (no capture loops, no in-tracker
  dispatch). Do not un-prune.
- ID injection exists: `Index::build_with_options(root, ScanOptions::write_ids())`
  (crates/core/src/index.rs:110) injects `[id]` into tracked markers via `write::inject`
  and returns `ScanReport { written_paths }`. Callers: daemon `build_index`
  (crates/indiana/src/daemon.rs:37) on every debounced rebuild, and CLI scan/copy.
  Capture piggybacks here.
- `::action`/`::todo` are today one tracked kind under two tokens (markers.rs ~139;
  `lookup()` matches `long` or `shorts`, so `task` can be an alias token).
- Casablanca: chokidar already watches `.indiana/**/*.md` → `TREE_CHANGED` push = free
  live refresh. CLI via `execFile` + binary probing in `main/lib/indiana.ts`. IPC
  pattern: `shared/ipc.ts` → `shared/domain.ts` → `main/ipc.ts handle()` →
  `preload/index.ts`. `MarkerComposer.tsx` is the reusable `::` picker.
- CLI pattern to copy: `CasablancaCmd`/`casablanca.rs` and `todos.rs`; per-subcommand
  `--json`; root helper `todo_root` (rename to `resolve_root`).
- `.indiana/context-model/CONTEXT-MODEL.md` §2/§6 already routes tasks/TODOs to
  chief-of-staff. Its `log.md` = agent-written knowledge journal; the new
  `chief-of-staff/log.md` = daemon/CLI-written run record. Parallel, never cross-written.

---

## Task A — Docs (vision + spec)

1. Rewrite `docs/chief-of-staff/COS_VISION.md`: drop the "unbaked" blockquote. Sections:
   What it is (attention-tracking, two queues) / Decided (queue mapping; tracker is
   hand-editable markdown; todos.db retired) / Direction / Non-goals (menulet surface,
   dispatch-from-tracker).
2. New `docs/chief-of-staff/COS_PRD.md` (the file COS_VISION promises): Intent / Files
   (tasks.md + log.md line grammars — the spec is the contract) / Capture & reconcile
   lifecycle / Queues / Faces (CLI, Casablanca panel) / Relationship to context-model
   log / Guarantees (write discipline, re-capture semantics, truncatable log) /
   Decided / Non-goals. Frontmatter per AGENT_WRITING.
3. Touch: `docs/indiana/IN_PRINCIPLES.md:19` (swap todos.db carve-out for
   tasks.md/log.md — keep line-neutral, file is at its `max_lines` cap);
   `docs/indiana/IN_FOLDER.md` layout + todos.db paragraph (~84-89);
   `docs/indiana/IN_COMMANDS.md` (`task` alias, queue note);
   `docs/casablanca/CASABLANCA_PRD.md` (panel bullet).

## Task B — Rust core (store, capture, templates)

**Formats** (`.indiana/chief-of-staff/`, both files carry frontmatter, one line = one thing):

`tasks.md` — queue = `## Agent` / `## Human` section; moving a line between sections
reassigns the queue (deliberate hand-edit affordance). Line grammar:

```
- [<state>] [<id>] <text> [(path:line)] [YYYY-MM-DD]
    states: ' ' open · '>' working · 'x' done · '!' failed
    regex: ^- \[([ >x!])\] \[([a-z][a-z-]*[a-z0-9])\] (.*)$
```

`[id]` = the id injected into the source marker (join key); `(path:line)` = capture-time
origin hint (optional; human-added rows have none). Unparseable lines are preserved
byte-for-byte on rewrite.

`log.md` — append-only, machine-written, declared truncatable:

```
YYYY-MM-DD HH:MM <event> [<id>] <detail…>
    events: capture · claimed · done · failed · resolved · task-add · task-done
```

**Code:**

1. `crates/core/src/markers.rs`: add `pub enum Queue { Agent, Human }`; `MarkerSpec`
   gains `queue: Option<Queue>` (Todo → Agent, Action → Human, others None); Todo
   `shorts: &["td", "task"]` — `::task` is an alias token, not a new Kind (no new
   template/counts/parser surface, table stays 11 rows). Tests: `lookup("task")`,
   `test_queue_split`; `test_only_action_todo_tracked` unchanged.
2. New `crates/core/src/cos.rs`: `tracker_path`, `log_path`, `Task`, `TaskState`,
   `load_tasks`, `append_task` (creates file with seed frontmatter if missing, appends
   under the right section), `set_task_state`, `append_log` (O_APPEND single write),
   `capture_and_reconcile(root, &Index) -> CaptureReport`. Write discipline mirrors
   write.rs: mtime snapshot → touch only the target line/append → `atomic_write` →
   one retry. Unit tests: grammar round-trip, unknown-line preservation, mtime guard.
3. Capture + reconcile, called at the end of `Index::build_with_options` when
   `!options.read_only`:
   - capture: every `spec.tracked && m.id.is_some()` whose id is absent from tasks.md →
     append to its queue with `(relpath:line)` + date; log `capture`. Id-presence is
     the double-capture guard (ids mint once).
   - reconcile: tracker row whose origin id vanished from the index → `[x]` + log
     `resolved` (marker removal = done, the Indiana convention); source status
     `:working` → `[>]`, `:failed` → `[!]`. Human-added rows (no origin) never touched.
   - deleted tracker row with live marker re-captures next scan — correct, source is
     truth.
   - tracker writes join `ScanReport.written_paths` so the daemon's `OwnWriteTracker`
     suppresses them.
   - integration tests in crates/core: correct queues, idempotent second build,
     marker-removed → `[x]` + resolved.
4. Templates (`crates/core/templates/`): chief-of-staff seeds `tasks.md` + `log.md`
   (skeletons above, empty sections); delete `actions.md` from the scaffold (existing
   roots keep theirs — refresh never deletes); update README seed + scaffold list in
   `crates/core/src/templates.rs` + tests. Prompt wording: `todo/prompt.md` → task is
   tracked in tasks.md (Agent queue), do it, mark `- [x]` (or `indiana task done <id>`),
   delete the marker line; `action/prompt.md` → human-queue item, do not execute.
   No `indianas/task/` folder (alias resolves through todo). Frontmatter untouched →
   `test_embedded_templates_match_marker_table` unaffected.

## Task C — CLI

Delete `TodoCmd` + `crates/indiana/src/todos.rs` + `tests/todo.rs` + rusqlite from
`crates/indiana/Cargo.toml`. Existing `todos.db` files become inert orphans (COS_PRD
notes: delete freely). New, following the `CasablancaCmd` pattern:

```
indiana task add   [--root R] [--queue agent|human] [--json] <text>   # default human: a typed add is operator intent
indiana task list  [--root R] [--queue …] [--state open|working|done|failed|all] [--json]  # default open+working
indiana task done  [--root R] [--json] <id>
indiana log        [--root R] [-n N] [--json]                         # tail, default 20
```

JSON shapes: task `{id, text, queue, state, origin:{path,line}?, created}`; log entry
`{ts, event, id, detail}`. Tests `crates/indiana/tests/task.rs`: add/list/done
round-trip, queue/state filters, json shapes, done-not-found, hand-edited file
survives, capture-created row visible after `indiana scan`; log tail order + `-n`.

## Task D — Daemon/dispatch logging

`crates/indiana/src/dispatch.rs`: `cos::append_log` at `try_dispatch` (after successful
claim → `claimed`), `run_turn`/`run_group_turn` outcome match (`done`/`failed`),
`mark_failed` callers (`failed`). Record tracker path in `OwnWriteTracker` after any
daemon-side cos write. Deferred: question/answer events, `indiana copy` logging.

## Task E — Casablanca task & action viewer

1. Placement: repo-scoped right aside in `Shell.tsx` — new
   `renderer/src/cos/TasksPanel.tsx` (w-80, `border-l border-pane-border bg-pane`),
   toggled from a TopBar button next to the daemon dot. Shows Human queue, Agent queue,
   recent activity (last ~15 log lines); row idiom from `history/HistoryPanel.tsx`.
2. Reads via CLI `--json` (core computes, faces render — grammar exists in one
   language): `listTasks(vault)` / `tailLog(vault, n)` in `main/lib/indiana.ts` calling
   `indiana task list --root … --state all --json` / `indiana log --json`; missing
   binary degrades like `copyAllMarkers`.
3. IPC: `COS_TASKS`, `COS_LOG`, `MARKER_APPEND` in `shared/ipc.ts`; `CosTask`,
   `CosLogEntry` types in `shared/domain.ts`; three `handle()`s in `main/ipc.ts`;
   `api.cos.tasks() / api.cos.log() / api.markers.append(rel, text)` in preload.
4. Live refresh: subscribe to existing `TREE_CHANGED` push (chokidar already covers
   `.indiana/chief-of-staff/*.md`), re-fetch on fire. No new watch plumbing.
5. Add-flow ("natively add indianas"): panel footer hosts `MarkerComposer` with
   `[todo, task, action]` options; submit → `MARKER_APPEND` appends `::<token> <msg>`
   to the currently open note through the existing note-write path (buffer/autosave
   stay coherent); daemon scan injects id → captures → watcher fires → row appears.
   No note open → composer disabled with hint; target-file picker deferred.
6. Act on task: click a row with origin → open origin note (existing note-open flow).

## Task F — Cone-shaped-tree architecture skill

New `.claude/skills/cone-tree-architecture/SKILL.md`, format of
`create-indiana-command` (frontmatter `name` + trigger-rich `description`; triggers:
"domain modelling", restructuring docs/, `.indiana/context-model/`, "where does this
file/fact go", adding/moving docs). Body = the cone laws distilled from
CONTEXT-MODEL.md + MENTAL_MODEL.md (link, don't restate, where a home exists):

- One apex: a single entry point (schema/index/purpose); depth 1 is the domain model;
  deeper = more specific, less stable.
- The tree widens downward — violations: a bulge (level narrower than the one below),
  deep single-child chains, a flat root with dozens of siblings.
- One home per fact (SSOT); everywhere else a link. Promote, never fork.
- Stability gradient: conflicts resolve toward the more stable level; knowledge flows
  up only by compression/promotion.
- Frontmatter on every file; one line = one thing; one index line per file; routed
  reads, never tree scans.
- Pre-flight checklist for adding/moving a file: which level, which parent, does the
  index know it, does anything it restates already have a home.

FUNDAMENTALS.md itself stays untouched (user's in-progress skeleton); the skill cites
it as the tier map.

## Sequencing (each step compiles + tests green)

1. B1 marker table → 2. B2 cos module → 3. B3 capture/reconcile → 4. B4 templates →
5. C CLI (+ todos removal) → 6. D dispatch logging → 7. A docs → 8. E Casablanca →
9. F skill. This repo's dogfood `.indiana/` gets the new seeds via
`indiana templates refresh .` after merge.

## Verification

- `cargo test -p indiana-core -p indiana` (queue tests, cos grammar, capture
  idempotence, templates, tests/task.rs; todo.rs removed;
  `test_embedded_templates_match_marker_table` green). `cargo build` proves rusqlite
  removal is clean.
- Manual CLI in a tmp repo: `indiana add .`; write `::task do x` in a note;
  `indiana scan .` → id injected, Agent row in tasks.md, `capture` in log.md; second
  scan → no duplicates; delete the marker line, scan → row `[x]` + `resolved`;
  `indiana task add --queue human "review y"`; `task list --json`; `task done <id>`;
  `indiana log -n 5`.
- Casablanca (Electron window, not :5173): `npm run typecheck && npm test`; then
  `npm run dev` — panel toggles, queues render, composer adds `::todo z` to the open
  note, row appears live after the daemon scan, click row → origin opens.
- Daemon loop check: with `indiana serve`, a tracker write causes no rebuild storm
  (own-write suppression) and one panel refresh per change.

## Risks / accepted tradeoffs

- Reconcile flips `[x]` when a whole source file is deleted (gone ≠ done). Accepted v1,
  documented in COS_PRD.
- Concurrent tracker writes (daemon vs CLI vs editor): mtime guard + one retry; worst
  case a write reports Retry and the next scan reconverges.
- `indiana todo` removal is breaking — sweep `.indiana/` dogfood + rules/ for
  references during implementation.
- `::action` wording change alters pasted-payload behavior — user approved.
- Deferred: dispatch-from-panel, copy-event logging, log rotation, Q/A log events,
  menulet surface, add-without-open-note, `--kind` filter narrowing for todo/action.
