---
status: draft
purpose: Contract for the Chief of Staff task tracker and action log — files, line grammars, capture/reconcile lifecycle, faces.
approval: pending
max_lines: 90
---

# COS_PRD — task tracker + action log

> Vision: [COS_VISION.md](COS_VISION.md). Engine invariants: [IN_PRINCIPLES.md](../indiana/IN_PRINCIPLES.md). Folder layout: [IN_FOLDER.md](../indiana/IN_FOLDER.md). Auto-run lifecycle: [IN_AUTORUN.md](../indiana/IN_AUTORUN.md).

## Intent
- One repo-local, hand-editable, token-efficient store for tasks and a durable record of what ran.
- Tasks are captured from markers; the marker line in source stays the single truth.
- Both files live in `.indiana/chief-of-staff/`; both carry frontmatter; one line = one thing.

## Files
- `tasks.md` — the tracker. Queue = the `## Agent` / `## Human` section a line sits under; moving a line between sections reassigns the queue (deliberate hand-edit affordance).
- Task line grammar: `- [<state>] [<id>] <text> [(path:line)] [YYYY-MM-DD]`.
- States: ` ` open · `>` working · `x` done · `!` failed.
- `[id]` is the id injected into the source marker — the join key. `(path:line)` is a capture-time jump hint, not maintained. Trailing date = creation. Both optional; hand-added rows have neither.
- Lines that do not match the grammar (or sit outside a queue section) are not tasks; machine rewrites preserve them byte-for-byte.
- `log.md` — the action log. Append-only, machine-written, safe to truncate or delete (history only).
- Log line grammar: `YYYY-MM-DD HH:MM <event> [<id>] <detail…>` (UTC).
- Events: `capture` · `claimed` · `done` · `failed` · `resolved` · `run` · `task-add` · `task-done`.
- `runs/<date>-<time>-<job>.md` — one durable audit record per agent turn, structured-first: the facts (job, outcome, timestamps, markers, per-turn tokens from the prompt response's `usage`, context window and cumulative cost from `usage_update`) live as YAML frontmatter; the markdown body carries the transcript (which otherwise dies with the job). Machine-written at turn end, timestamped so a retried id keeps every attempt; pruned to the newest 200, safe to delete.
- The record grammar exists in one language: `run_record.rs` writes and parses; `indiana runs [-n N] [--json]` is the face surface. Faces never parse record files themselves.
- The `run` event indexes the record: `run [<job>] <outcome> · <tokens> · <cost> · <record path>`, usage parts present only when reported.

## Capture and reconcile
- Runs only at deliberate entry points (`ScanOptions.capture`): daemon rebuilds and explicit `indiana scan <path>`. A plain write scan or `indiana copy` injects ids and nothing else — no folder is turned into a chief-of-staff root as a side effect. `.indiana/` itself is never scanned ([IN_SCAN.md](../indiana/IN_SCAN.md)), so tracker-internal markers are inert and capture cannot loop.
- A whole pass is at most one tracker rewrite plus one log append — a burst of new markers never fans out into N writes.
- Capture: a tracked marker (`::todo`/`::task` → Agent, `::action` → Human, per the marker table) whose id is absent from tasks.md gets a tracker line with origin and date, plus a `capture` log line. Id presence — in the file or earlier in the same pass (a copy-pasted bracket line) — is the double-capture guard.
- Reconcile: a captured task whose id vanished from source flips `[x]` (`resolved` log line) — marker removal is the done convention. A file that failed to read merely hides its markers; its tasks are left alone. A live marker's `:working`/`:failed`/`:done` bracket mirrors onto the task. A bare marker never flips a task back: human state edits stand.
- Deleting a captured tracker row while its marker lives re-captures next scan — source is truth. Dismiss by resolving the marker, not the row.
- Hand-added tasks (no origin) are never touched by reconcile.
- Known tradeoff: deleting or gitignoring a whole source file resolves its tasks as `[x]` (gone ≠ done; indistinguishable from marker removal).

## Faces
- CLI: `indiana task add [--queue agent|human]` (default human — a typed add is operator intent), `indiana task list [--queue] [--state open|working|done|failed|all]` (default open+working), `indiana task done <id>`, `indiana log [-n N]`, `indiana runs [-n N]`. Each takes `--root` and `--json`. Only a confirmed write reports success: a write that loses the mtime race twice fails with "tracker is busy" instead of printing a phantom id.
- Daemon: dispatch lifecycle appends `claimed`/`done`/`failed` for auto-run and group turns.
- Casablanca: the tasks panel renders both queues and the recent log, refreshes only when tasks.md/log.md actually change, and jumps to a task's origin. It reads through the CLI `--json` — core computes, faces render. The composer appends markers through the live editor (one writer per open note), never straight to disk beneath a dirty buffer.
- Casablanca: the Agent runs panel (top-right bolt icon) lists `runs/` records newest first — outcome, tokens, cost — through `indiana runs --json`, and shows the selected record's transcript read raw for display. No daemon involved, so history is browsable while it is offline.

## Guarantees
- Machine writes go read → mtime-guard → single-line change or append → atomic replace → one retry — the write.rs discipline. Worst case a write retries on the next scan; nothing is lost.
- All tracker/log writes made during a scan report into `ScanReport.written_paths`; the daemon records them as own writes so its watcher does not rebuild-storm.
- Relationship to the context-model journal: `.indiana/context-model/log.md` is agent-written knowledge (what was learned, via prompts); `.indiana/chief-of-staff/log.md` is machine-written operations (what ran). Same filename, parallel on purpose, never cross-written.

## Decided
- Markdown line formats over JSONL/SQLite: the operator reads and edits the tracker directly; token cost stays low.
- `::task` is an alias token of `::todo` — one tracked agent-queue kind, no new grammar row.
- `::action` compiles as a human-queue item; agents are told not to execute it.
- `todos.db` and `indiana todo` retired. Stale `todos.db` files are inert; delete freely.

## Non-goals (v1)
- Dispatch-from-tracker or from the panel; log rotation; copy-event and question/answer log events; menulet surface.
