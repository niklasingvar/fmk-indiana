---
status: draft
purpose: Step-by-step runbook to implement Indiana end to end, sequenced into small verifiable steps.
approval: pending
---

# Indiana — implementation runbook

## Context
- Indiana is specified, not built. The repo is pure markdown spec ([PURPOSE.md], [GOAL.md], [PHASES.md], `INDIANA/IN_*.md`).
- Goal: build the whole product, but in many small steps each with a verify gate, so spec stays the contract and code conforms ([IN_PRINCIPLES.md]: spec is the contract).
- Language/shape are fixed by the specs: Rust, one static `aarch64-apple-darwin` binary `indiana`, multi-mode, daemon + Unix socket, faces (CLI/MCP/menulet) as dumb clients.
- Sequencing mirrors [PHASES.md] (scan → counts+watch → copy → MCP → menulet → casablanca). Each milestone below ships something runnable.
- Tests are pre-mapped in [IN_TEST.md] (E1–E11). Each step names the E-criteria it satisfies; "drift is a missing test."

## Decisions taken this session (override committed specs)
- D1 — On-disk id format is the bracket form `::action[id:status] msg`, not the HTML-comment tail. IN_LINE.md is rewritten in M0. Rationale: markers are visible source annotations anyway; the "invisible in rendered markdown" reason for the HTML comment never applied to the marker itself.
- D2 — IDs are fake pronounceable syllable tokens (`frata-nimta`, `lurvo-pannik`), pattern `[a-z]+-[a-z]+(-[0-9]+)?`, generated, not a real-word dictionary. IN_IDENTITY.md is updated in M0. Effectively unlimited pool; no bundled word lists.
- D3 — Plan steps are a plain numbered checklist (this doc). Indiana's own markers are not used to track them.
- D4 — Defaults chosen without asking (flag if wrong): `indiana scan` prints a human-grouped list and accepts `--json`; socket protocol is line-delimited JSON via serde; walker uses the `ignore` crate with gitignore disabled (walk all markdown, only exclude `.indiana/` and non-`.md`).
- D5 — Any scan injects ids, not just the daemon. `indiana scan <path>` mutates on first sight too (per IN_IDENTITY). Consequence: scan is no longer read-only — a plain scan can rewrite `::action`/`::todo` lines. Dogfood stays safe only because this repo's markers sit in code spans. `--read-only` is the escape hatch.
- D6 — E6 write-path tests use real temp dirs (assert on bytes), matching existing test style and exercising the real OS rename/fsync path. No FS abstraction.
- D7 — Malformed brackets are repaired, not trusted. A bracket whose id fails `[a-z]+-[a-z]+(-[0-9]+)?` gets a fresh id; an unknown status word is dropped to open. Must stay idempotent (a repaired line rescans clean). More writes through the chokepoint — accepted.
- D8 — MCP transport is stdio JSON-RPC (`indiana mcp`), which dials the Unix socket for data. Standard for MCP clients; the socket stays the daemon's data plane.
- D9 — Scope section rule follows IN_SCOPE.md, not the stale E4 table wording: inline markers on headings stay inline; a marker alone immediately before an ATX heading scopes that section.
- D10 — MCP is a small manual stdio JSON-RPC face for now, not `rmcp`. Rationale: no async runtime or large dependency until the MCP surface is stable.
- D11 — M12/M13 are separate sub-product planning milestones here. Added runbooks; did not invent a Tauri app or Casablanca renderer in the same pass.

## Architecture (target)
- Workspace, two crates to start:
  - `crates/core` (`indiana_core`): marker table, line parser, walker, index model, id generator, write chokepoint, scope resolver, compiler, prompt templates, config types. The core computes everything.
  - `crates/indiana` (bin `indiana`): arg dispatch, daemon, socket server + client, watch, MCP face, `service install`. Faces only render.
- `menulet/` (Tauri app) added at M12; `casablanca` at M13.
- One marker table drives parser + compiler + identity ([IN_PRINCIPLES.md]: one table). Prompt wording lives in `crates/core/prompts.toml`, embedded via `include_str!` ([IN_PRINCIPLES.md]: content is data).
- One write function is the only mutator of user files ([IN_PRINCIPLES.md]: single chokepoint).
- Recommended deps: `ignore` (walk), `notify` + `notify-debouncer-full` (~300 ms watch), `serde`/`serde_json`, `tempfile` (atomic write), `arboard` (clipboard), `toml` (prompt data), `clap` (arg parse). MCP face: prefer `rmcp` (official Rust MCP SDK) — if it pulls `tokio`, isolate async to the MCP module only; daemon core stays sync-threaded.

---

## Progress (as of 2026-06-22)
- DONE M0 — specs reconciled: IN_LINE.md (bracket form), IN_IDENTITY.md (pronounceable ids), IN_TEST.md (E1/E6 + new inline-code row), IN_SCAN.md (inline-code-span rule). No stale `<!--in:-->` / dictionary refs remain.
- DONE M1 — cargo workspace builds; release `aarch64-apple-darwin` is a 302k single binary linking only libSystem.
- DONE M2 — marker table + parser; E1/E2/E3 covered.
- DONE M3 — walker (`ignore`, excludes `.indiana/` + `.git/`), `Index`, `indiana scan [path] [--json]`; E5 + E10 covered. 30 tests green; `indiana scan .` on this repo = 0 markers.
- Discovered gap (fixed): fence-only suppression missed inline code spans, which the specs use everywhere → ~30 false positives on self-scan. Added CommonMark backtick-run code-span suppression to parser + IN_SCAN.md.
- DONE M4 — daemon (`indiana serve`) binds `~/.indiana/indiana.sock`, holds index in memory, line-delimited JSON protocol, stale-socket recovery, `config.json` + `indiana add`, `scan` as socket client w/ standalone fallback. `INDIANA_HOME` env overrides dir for tests. E8 covered. 34 tests green; live daemon round-trip verified. Phase 1 (PHASES.md) complete.
- Commits: docs spec set → .gitignore → CLAUDE.md(commit-often) → feat M0-M3 → feat M4. Docs and code in separate commits per new CLAUDE.md rule.
- DONE M5 — daemon watches roots (notify + debouncer ~300ms); each coalesced batch rebuilds the held index. E11 covered (4 tests).
- DONE M6 — Index::counts() per-kind tallies in core; scan prints summary. Phase 2 (PHASES.md) complete.
- DONE M7 — identity + write chokepoint. `Index::build` injects/repairs tracked ids through `core/src/write.rs`; `Index::build_read_only` and `indiana scan --read-only` suppress writes; daemon records own writes and suppresses matching watch events. 59 tests green.
- DONE M8 — scope resolution in `core/src/scope.rs`; index stores `{scope kind, content}` per marker. E4 covered.
- DONE M9 — shared compiled payload model, embedded `prompts.toml`, `indiana copy`; copy renders the same payload MCP will expose.
- DONE M10 — `indiana mcp` stdio JSON-RPC face; daemon socket now serves compiled payload.
- DONE M11 — `indiana service install` writes launchd plist; release `aarch64-apple-darwin` build verified.
- DONE M12 — menulet split into [MENULET_RUNBOOK.md](MENULET/MENULET_RUNBOOK.md). App build deferred to that runbook.
- DONE M13 — Casablanca split into [CASABLANCA_RUNBOOK.md](CASABLANCA/CASABLANCA_RUNBOOK.md). Renderer build deferred to that runbook.
- Deviations to confirm: walker also prunes `.git/` (spec named only `.indiana/`); reactions silently drop trailing text (e.g. `::h foo` → no message); daemon serves cwd when config empty and no root arg; mtime guard exists but deterministic race coverage is still shallow; MCP currently implements the required JSON-RPC surface manually rather than through `rmcp`.

---

## M0 — Reconcile specs to this session's decisions
Do this first: code conforms to spec, so fix the spec before writing code.

1. Rewrite `INDIANA/IN_LINE.md` to the bracket form (D1) → verify: examples read `::action[id]` / `::action[id:done]` / `::todo[id:failed]`; "Why an HTML comment" section replaced with "Why brackets" (id rides the command token, not a line-end tail; parser strips `[...]` between token and message); keep idempotent/atomic/travels-with-line discipline.
2. Update `INDIANA/IN_IDENTITY.md` to the pronounceable generator (D2) → verify: "Format" line describes generated syllable tokens, pool note changed from "~158k pairs" to "generated, effectively unlimited"; collision rule (`-2`, `-3`) retained.
3. Ripple to `INDIANA/IN_TEST.md` → verify: E1 `test_marker_tail_stripped` and all E6 rows use `::action[id:status]` not `<!--in:...-->`.
4. Skim `INDIANA/IN_COMMANDS.md` and `INDIANA/IN_SCAN.md` for stale tail references → verify: links to IN_LINE still hold, no contradicting on-disk syntax remains.
5. Set frontmatter `approval` on the touched files appropriately → verify: every edited markdown still has valid YAML frontmatter ([CLAUDE.md] rule).

## M1 — Scaffold (Phase 1 base)
6. `cargo new --lib crates/core` + `cargo new crates/indiana`; root workspace `Cargo.toml` → verify: `cargo build` succeeds, `indiana` bin links `indiana_core`.
7. Add release profile for a static `aarch64-apple-darwin` build per [DISTRO.md] → verify: `cargo build --release --target aarch64-apple-darwin` produces one binary, no runtime deps.
8. Create the test harness: `scan_fixture(dir) -> Index` helper + `crates/indiana/tests/fixtures/` ([IN_TEST.md] structure) → verify: an empty fixture returns an empty `Index`; `cargo test` runs.

## M2 — Marker table + line parser (Phase 1 — E1, E2, E3)
9. Declare the one marker table in `core/src/markers.rs`: rows of `{short, long, kind, takes_msg, tracked}` from [IN_COMMANDS.md] table → verify: unit test asserts 9 kinds, short/long both resolve ([IN_TEST.md] `test_marker_long_form`).
10. Implement `parse_line(line, &mut FenceState) -> Option<Marker>` — column-0 or inline-at-end, strip `[id:status]`, split message ([IN_COMMANDS.md] syntax, D1) → verify: E1 `test_marker_column_zero`, `test_marker_inline`, `test_marker_with_message`, `test_marker_tail_stripped`.
11. Ambiguous-line rule: ≥2 markers on a line → skip + warn ([IN_SCAN.md]) → verify: E1 `test_marker_ambiguous_line`.
12. Fence tracking: independent ``` ``` ``` and `~~~` states, leading-`---` YAML frontmatter special case, indented-`::` still resolves by column rule ([IN_SCAN.md] code fences) → verify: E2 `test_fence_backtick`, `test_fence_tilde`, `test_fence_independent`, `test_fence_unclosed`, `test_fence_yaml_frontmatter`, `test_indented_ignored`; warn on EOF inside open fence.
13. Statelessness: `parse_line` pure given line + fence state ([IN_PRINCIPLES.md]) → verify: E3 `test_parse_line_pure` (property test), `test_one_marker_per_line`.

## M3 — Walker + index + `indiana scan` one-shot (Phase 1 — E5, E10)
14. `core/src/walk.rs`: walk markdown under a root via `ignore` (gitignore disabled, D4), exclude `.indiana/`, skip non-`.md` → verify: E5 `test_full_walk`, `test_exclude_indiana_dir`, `test_skip_non_markdown`.
15. `Index` model: per-marker `{path, line, kind, message, id?, status?}` ([IN_PRD.md] decided: path+line+id travel) → verify: `scan_fixture` populates these fields.
16. `indiana scan [path]` standalone command: walk + parse, print human-grouped list, `--json` flag (D4) → verify: E10 `test_cli_scan`; running against this repo prints zero markers (all `::` in specs sit in fences/tables — real fence regression check).

## M4 — Daemon + socket lifecycle + config (Phase 1 — E8)
17. `indiana serve`: bind `~/.indiana/indiana.sock`, hold the scanned `Index` in memory ([IN_DAEMON.md]) → verify: starts, binds, holds a scan of the configured root.
18. Stale-socket recovery: connect-first, refused → unlink+bind, alive → exit "already running" ([IN_DAEMON.md]) → verify: E8 `test_socket_single_bind`, `test_stale_socket`.
19. `~/.indiana/config.json` monitored-folders list (input, not derived — [IN_DAEMON.md] carve-out) → verify: E8 `test_config_persists`.
20. Socket protocol (line-delimited JSON, D4) + make `indiana scan` able to run as a socket client of a live daemon (falls back to standalone if none) → verify: E8 `test_client_reconnect`; daemon and CLI agree on counts.

## M5 — Watch (Phase 2 — E11)
21. FSEvents via `notify` + debouncer (~300 ms), per-path rescan not global ([IN_SCAN.md] walk and watch) → verify: E11 `test_watch_new_file`, `test_watch_modify`, `test_watch_delete`, `test_watch_debounce`.
22. Full walk on startup, event-driven after; index rebuildable ([IN_SCAN.md] decided) → verify: E9 `test_source_is_truth` (delete index, rescan, byte-identical).

## M6 — Counts (Phase 2)
23. Per-kind tallies on the index (actions, todos, notes, fixes, elaborates, questions, hate, love, keep) computed in core ([IN_PRD.md] copy and counts) → verify: count test on a mixed fixture; faces read the count, never compute it (E9 `test_faces_never_compute`).

## M7 — Identity + write chokepoint (the dangerous milestone — E6, E9)
> First milestone that writes to user markdown. Decisions: D5 (any scan injects), D6 (real temp dirs), D7 (repair malformed). Build the write path behind one function so the blast radius is one file, one contract.

24. Spec ripple (do first, docs only): record the now-decided opens →
    - `IN_LINE.md` Open → decided: repair malformed brackets (D7).
    - `IN_SCAN.md` → note read-only-by-default has one write (injection), and that any scan (not just the daemon) may perform it (D5).
    - `IN_TEST.md` E6 Open → real temp dirs (D6).
    - `IN_IDENTITY.md` → pin the generator scheme (step 25).
    - verify: specs current, frontmatter valid.
25. Id generator `core/src/id.rs` (D2): syllable scheme = `CV(C)` syllables joined `syl-syl`, lowercased, matching `[a-z]+-[a-z]+(-[0-9]+)?`. Randomness: prefer a tiny dep (`fastrand`) over hand-rolled RNG. Collision within a scan → append `-2`, `-3`; track ids seen this scan. → verify: pattern test, `test_id_uniqueness` (no reuse within a scan), determinism not required.
26. The one write function `core/src/write.rs` — the single chokepoint ([IN_PRINCIPLES.md]). Contract:
    - Byte-preserving: edit only the target line; preserve EOL style (LF/CRLF) and final-newline presence. Operate on raw bytes / line byte-ranges, not `String::lines()` (which drops EOLs).
    - Atomic: temp file in the same dir → write → `fsync` → `rename` over original (`tempfile::NamedTempFile::persist`).
    - mtime-guard: stat before read; re-stat before rename; if changed → abort, return `Retry`, re-queue a fresh single-file rescan ([IN_SCAN.md] concurrency).
    - Idempotent: a line already carrying a valid bracket is left byte-identical.
    - → verify: E6 `test_write_atomic`, `test_mtime_guard`, `test_id_idempotent`; E9 `test_write_path_single_function` (grep: the only `fs::write`/rename of *user markdown* lives here — `config.json` writes in `config.rs` are not user files and are exempt).
27. Injection + repair pass, wired into `Index::build` (D5). For each `::action`/`::todo`:
    - no bracket → inject a fresh `[id]`.
    - bracket with id failing the pattern → repair: replace with a fresh `[id]` (D7).
    - bracket with unknown status word → drop status to open (D7).
    - valid bracket → leave it.
    - → verify: E6 `test_id_first_injection`, `test_id_only_tracked` (reactions/`::fix` etc. never get one), `test_status_done`; new `test_bracket_repair_*`. Add a `--read-only` flag to suppress writes (D5 escape hatch) → `test_scan_read_only_no_write`.
28. Own-write suppression ~500 ms ([IN_SCAN.md] own writes). Correctness already holds via idempotency (a self-write triggers one rebuild that re-injects nothing); suppression is the optimization that avoids that extra rebuild. Mechanism: chokepoint records `(path → instant)`; the M5 watcher skips a path within its window. → verify: E6 `test_suppress_own_write` — needs a rebuild counter exposed for the test; if flaky, mark `#[ignore]` per [IN_TEST.md] Open.

## M8 — Scope resolution (Phase 3 prep — E4)
> [IN_SCOPE.md] read. New module `core/src/scope.rs`: a second pass over a file's lines given each marker's line/column (the per-line parser can't see spans). Resolve at scan time, freeze onto the marker. Build order: inline + next-row, then section, then range.

29. Inline + next-row (clearest first): inline = marker at EOL targets that line's content (minus marker + bracket); next-row = marker alone on a line targets the next contiguous non-blank block until a blank line (whole list/blockquote is one block). Spans never cross EOF. → verify: E4 `test_scope_inline`, `test_scope_next_row`, `test_scope_file_bound`.
30. Section (needs heading-level tracking): marker alone immediately before an ATX (`#`) heading targets until the next equal-or-higher ATX heading; Setext underline is not an anchor; inline-on-heading stays inline; nested `###` gets its own narrow span. Most-specific wins — an inline marker inside a section keeps its own span (falls out naturally since each marker resolves by its own position). → verify: E4 `test_scope_section`, `test_scope_most_specific`.
31. Range deferred ([IN_SCOPE.md] decided): `::end` not implemented; must not crash. → verify: E4 `test_scope_range_deferred`. Store `{scope_kind, scope_content}` on the located marker for the payload.

## M9 — Compiler + copy bundle (Phase 3 — E7, E10)
> One compiled-payload model feeds both copy and MCP ([IN_ARCHITECTURE.md], [IN_MCP.md]). Copy renders text; MCP returns structure.

32. Prompt templates `core/prompts.toml`, embedded via `include_str!`, parsed with `toml` — wording is data, keyed by kind incl. the no-message `::question` variant ([IN_PRINCIPLES.md] content is data, [IN_COMMANDS.md] compiled prompts). → verify: E7 `test_prompt_templates_external` (changing wording touches only the toml, not engine logic).
33. Compiler `core/src/compile.rs`: `(marker + resolved scope) → CompiledMarker { id, kind, raw_token, compiled_prompt, message?, path, line, scope_kind, scope_content, status? }` — the shared payload model. → verify: E7 `test_compile_hate`, `test_compile_fix`, `test_compile_question`, `test_scope_in_bundle`, `test_copy_all_commands`.
34. `indiana copy [path]`: render the payload to clipboard via `arboard`; same daemon-or-standalone resolution as `scan` ([IN_PRD.md]). → verify: E10 `test_cli_copy`.

## M10 — MCP face (Phase 4 — D8)
> `indiana mcp`: a stdio JSON-RPC MCP server that dials the Unix socket for data. Thin renderer over the M9 payload — never parses, counts, or resolves scope ([IN_MCP.md]).

35. Add `indiana mcp` (stdio) JSON-RPC; keep daemon core sync. Daemon gains a socket request returning the compiled payload (extend `protocol.rs`). → verify: builds, no async runtime required yet.
36. MCP surface per [IN_MCP.md] Contract: list pending indianas; read one by id; read full payload; expose marker grammar/prompt meanings. Report scan status (never silently stale). Boundaries: never edits files; completion writes (if added) flow through the M7 chokepoint. → verify: agent lists/reads with id/kind/prompt/message/path/line/scope/status; payload shape matches `copy` source (E9 faces-render).

## M11 — Service install + packaging (cross-cutting — E10, DISTRO)
37. `indiana service install`: write `~/Library/LaunchAgents/<label>.plist` with `RunAtLoad` + `KeepAlive` pointing at `indiana serve` ([IN_DAEMON.md] crash recovery, [DISTRO.md]). → verify: E10 `test_cli_service_install` (valid XML, correct label + program path).
38. Release artifact + install-by-copy to `~/.local/bin/indiana` ([DISTRO.md] now/dogfood). → verify: fresh shell runs `indiana scan`.

## M12 — Menulet (Phase 5 — own planning pass)
> Large separate sub-project; needs its own breakdown + Tauri/JS toolchain before building.
39. Tauri app in `menulet/`: sidecar-bundles `indiana` as `Contents/MacOS/indiana`; connect-or-spawn the daemon; PATH detection by standard locations (not shell rc); add/remove monitored folders (via `config` + daemon); one-click copy. Shows, never computes ([MENULET_PRD.md], [DISTRO.md] sidecar). Visual tests live here, not in core ([IN_TEST.md]).

## M13 — Casablanca (Phase 6 — own planning pass)
40. Separate sub-product, unrelated to markers: agents emit Casablanca-formatted terse output; Indiana visualizes it ([CASABLANCA_PRD.md]). Write its own spec/runbook before building.

---

## Cross-cutting invariants (check at every milestone — E9)
- Source is truth: delete `.indiana/`, rescan, byte-identical state. No state outside source except `config.json`.
- One marker table; parser/compiler/identity never re-encode the set.
- Core computes, faces render: CLI/MCP/menulet only surface the core's payload.
- One write chokepoint: grep proves no other file write exists.

## End-to-end verification
- `cargo test` green across E1–E11 (skip-listed where [IN_TEST.md] Open marks watch tests flaky).
- `cargo build --release --target aarch64-apple-darwin` → single binary.
- Dogfood read-only: `indiana scan .` on this repo → zero markers (proves fences/tables/frontmatter are excluded).
- Dogfood write: drop a scratch `tmp.md` with `::action do thing` outside `.indiana/` → daemon injects `::action[<id>] do thing`; re-scan is byte-identical; delete `.indiana/` and rescan returns identical state.
- `indiana serve` + second `indiana serve` → clean "already running".
- `indiana copy` against a fixture → clipboard holds the compiled bundle.

## Open questions (carry into execution)
- Q1 — Phase boundary: M4 (daemon) was chosen as in-scope for the first ship even though watch/counts arrive later; confirm the first dogfood milestone is M4 not M3.
- Q2 — `IN_DAEMON.md` Open: do config edits hot-reload, or require restart? Pick at M19/config.
- Q4 — `IN_TEST.md` Open: E11 watch tests in CI vs manual.
