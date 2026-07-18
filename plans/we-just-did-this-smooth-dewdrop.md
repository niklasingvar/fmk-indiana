# Agent job visibility in Casablanca: spinner, live marker claim, follow chat

## Context

The auto-run loop (`::fix -a` → daemon claims → ACP turn → commit) now works, and the
uncommitted `note:changed` adoption fix stops stale buffers from wiping claims. But the
loop is invisible and lossy in the editor:

- No prominent spinner for ongoing jobs (the TopBar chip's `◌` glyph is ~11px and
  effectively invisible when spinning).
- The open note only adopts the daemon's claim edit (`::fix -a` → `::fix[id:working] -a`)
  via a full Lexical remount — cursor/undo destroyed — and refuses when the buffer is
  dirty, which is exactly the stale-buffer double-dispatch bug's origin.
- There is no way to follow what the agent is doing: ACP `session/update` chunks are
  dropped in the pump loop (`crates/indiana/src/acp.rs:141-148`) and only appended raw
  to `~/.indiana/dispatch/<job-id>.log`.

Goal: (a) a visible spinner for ongoing jobs, (b) in-place marker id + inline `:working`
spinner in the open note without breaking editing, (c) clicking a spinner opens a small
chat popover streaming the agent's work, plus (d) a documentation pass so future
sessions can act on this subsystem without re-reading the code.

**Decisions confirmed with the user:**
- Chat transport: daemon-side transcript over the existing unix socket (new
  `jobtranscript` command, since-seq polling). Not log tailing.
- Transcript dies with the job (live follow view only; raw log stays on disk).

**Constraint:** build on, don't disturb, the uncommitted working-tree changes
(flag-preserving claims in `core/write.rs`, RootLease serialization in `dispatch.rs`,
hand-rolled watch debouncer in `daemon.rs`, `note:changed` adoption in Casablanca).

## What already exists (reuse, don't rebuild)

- Jobs feed end-to-end: daemon `Request::Jobs`/`AnswerJob` over `~/.indiana/indiana.sock`
  (`crates/indiana-protocol/src/lib.rs:113-155`, `daemon.rs:425-436`) → Casablanca socket
  client `crates/casablanca/src/main/lib/indiana.ts` (`agentJobs`, `answerAgentJob`) →
  IPC `INDIANA_JOBS`/`INDIANA_ANSWER_JOB` (`shared/ipc.ts:35-37`, `main/ipc.ts:176-184`,
  `preload/index.ts:89-95`) → `TopBar.tsx` 1s poll + job chips + `QuestionPopover`.
- Marker styling: `MarkerHighlightPlugin.tsx` `registerNodeTransform(TextNode)` with
  `splitText` boundary technique; regex already tolerates the `[id:status]` bracket.
- External-edit adoption: `useVault.ts:65-96` `note:changed` handler (own-echo /
  re-baseline / clean-adopt-remount / dirty-warn branches).
- Mock ACP adapter for tests: `crates/indiana/src/bin/mock_acp_agent.rs` (emits an
  `agent_message_chunk` "mock agent working"; `MOCK_ACP_MODE=question|fail` modes),
  test patterns in `crates/indiana/tests/autorun.rs` (`--features test-support`).

## Part 1 — Rust: transcript capture + `jobtranscript` protocol command

Do this first; everything in the UI polls it.

1. **`crates/indiana-protocol/src/lib.rs`**
   - `Request::JobTranscript { job_id: String, #[serde(default)] since_seq: u64 }`
     (wire tag `jobtranscript` via existing lowercase rename).
   - `TranscriptEventKind` (snake_case): `Agent | Thought | Tool | Question | Answer`.
   - `TranscriptEvent { seq: u64, kind: TranscriptEventKind, text: String }`.
   - `JobTranscriptResponse { found: bool, events: Vec<TranscriptEvent>, next_seq: u64 }`
     — `found: false` when the job id is unknown (finished/never existed); UI shows
     "turn ended".
   - Serde round-trip unit test next to the existing ones.

2. **`crates/indiana/src/acp.rs`** — surface `session/update`: in the `call()` pump loop,
   where notifications currently `continue` (line ~148), invoke a new `on_update`
   callback with `msg.get("params")` when the method is `session/update`. Thread
   `on_update: &mut U where U: FnMut(&Value)` through `run_turn`/`call` alongside the
   existing `on_elicitation` pattern; callers that don't care pass `&mut |_| {}`.
   Keep logging unchanged.

3. **`crates/indiana/src/dispatch.rs`** — transcript in `JobRegistry`:
   - `JobEntry` gains `transcript: Vec<TranscriptEvent>` + `next_seq: u64`.
   - `append_update(&self, id, params)`: parse `params.update.sessionUpdate`:
     `agent_message_chunk` → `Agent` (merge into previous event when it's also `Agent` —
     append text, keep seq — so chunk streams don't explode the buffer);
     `agent_thought_chunk` → `Thought` (same merge); `tool_call`/`tool_call_update` →
     `Tool` with `title` (fallback `toolCallId`) + `status`; else ignore. Cap at 1000
     events, drop from the front (seq keeps increasing).
   - `transcript(&self, id, since_seq) -> Option<(Vec<TranscriptEvent>, u64)>`.
   - In `await_answer`: append a `Question` event after parsing the question and an
     `Answer` event after `recv()` — the chat reads as a conversation with no UI
     special-casing.
   - `run_turn`/`run_group_turn`: pass `|params| jobs.append_update(id, params)` into
     `AcpAgent::run_turn`.
   - `Dispatcher::job_transcript(id, since_seq)` delegating to the registry. Transcript
     dies with `jobs.remove` at turn end (decided).

4. **`crates/indiana/src/daemon.rs`** — `Request::JobTranscript` arm in `handle()` next
   to `Request::Jobs`: map to `JobTranscriptResponse` (`found:false, events:[],
   next_seq:0` on `None`).

5. **Tests**: unit tests in `dispatch.rs` for chunk merging + cap. Integration test in
   `crates/indiana/tests/autorun.rs` (`test_job_transcript_follows_live_turn`): mock
   adapter in `question` mode → wait for `awaiting_input` via `{"cmd":"jobs"}` → send
   `{"cmd":"jobtranscript","job_id":…,"since_seq":0}` → assert an `agent` event
   containing "mock agent working" and a `question` event → answer via `answerjob` →
   assert resolution and that `jobtranscript` afterwards returns `found:false`.

## Part 2 — Casablanca plumbing (shared/main/preload)

1. **`shared/domain.ts`**: `TranscriptEventKind`, `TranscriptEvent { seq, kind, text }`,
   `JobTranscriptResult { found, events, nextSeq }`.
2. **`main/lib/indiana.ts`**: `jobTranscript(jobId, sinceSeq)` →
   `daemonRequest({ cmd: 'jobtranscript', job_id, since_seq })`, mapping `next_seq` →
   `nextSeq`; on socket error return `{ found: false, events: [], nextSeq: sinceSeq }`
   (offline daemon is ordinary, same stance as `agentJobs`).
3. **`shared/ipc.ts`**: `INDIANA_JOB_TRANSCRIPT: 'indiana:job-transcript'`;
   **`main/ipc.ts`**: invoke handler next to the existing indiana handlers;
   **`preload/index.ts`**: `indiana.transcript(jobId, sinceSeq)`.

## Part 3 — Live marker claim in the open buffer (no remount)

1. **New pure helper `shared/marker-claim.ts` (+ vitest `marker-claim.test.ts`)**:
   `diffMarkerClaims(oldBody, newBody): MarkerClaimPatch[] | null` where
   `MarkerClaimPatch { find: string; replace: string }` (exact full-line texts).
   Match only when: equal line counts; every differing line pair is identical except the
   new line inserts/replaces a `\[[a-z0-9-]+(?::(?:working|done|failed))?\]` bracket
   immediately after the `::<kind>` token, with flags + message byte-identical.
   Tests: fresh claim, working→failed, group claim (multiple lines), rejects content
   edits and line-count changes.

2. **`useVault.ts`** — new branch in the `note:changed` handler between "own echo" and
   "clean adopt" (line ~75): parse both bodies with `parseNoteDocument`, require
   identical frontmatter, run `diffMarkerClaims`. On match:
   - `setActiveNote(fresh)` (re-baseline) and `setDraft` with the same string patch
     applied to the draft body (autosave state correct before the editor exports).
   - Publish `markerPatch: { id: n, patches } ` state (incrementing id so repeated
     claims retrigger); expose from the hook. No `noteVersion` bump — no remount.
   - This branch runs for dirty buffers too — that is the point: the claim lands
     surgically while the user keeps typing elsewhere.
   - Autosave interplay: clean buffer → serialized draft equals new baseline → autosave
     no-ops; dirty buffer → autosave persists user text *including the claim* → claims
     are never clobbered → no more double dispatch.

3. **New `renderer/src/editor/plugins/MarkerClaimPlugin.tsx`** (register in
   `Editor.tsx`; `EditorPane.tsx` threads `vault.markerPatch` down):
   - `useEffect` on `patch.id` → `editor.update(() => {…}, { tag: 'history-merge' })`.
   - Per patch: walk root paragraphs, join contiguous plain TextNodes (reuse/export
     `contiguousTextSiblings` from `MarkerHighlightPlugin.tsx:103-126`), exact-match the
     joined text against `patch.find`, then `spliceText` at the offset right after
     `::<kind>` (deleteLen covers a replaced old bracket for the `:failed` case).
   - Selection safety: if a RangeSelection is anchored in the spliced node at an offset
     ≥ insertion point, shift by the length delta; otherwise untouched.
   - `history-merge` is load-bearing: the claim must not be its own undo step (Cmd+Z
     stripping a claim → autosave writes unclaimed marker → daemon re-dispatches).
   - No matching line (user edited that exact line unsaved): warn and no-op — degrades
     to today's dirty-diverge behavior.
   - `MarkdownPlugin` needs no change (the patch triggers export → `setDraftBody` with
     an identical body → no extra save).

4. **Inline `:working` spinner — extend `MarkerHighlightPlugin.tsx`**:
   - `INDIANA_MARKER_WORKING_STYLE = INDIANA_MARKER_STYLE + ' --marker-working: 1;'`
     (custom property as a CSS-selectable sentinel — no DecoratorNode, so markdown
     export stays byte-identical and editing inside the marker keeps working).
   - In `highlightMarker`: also match `/\[([a-z0-9-]+):working\]/` inside the marker
     text and give that bracket range the working style via the existing `splitText`
     offset walk. Introduce `isMarkerStyle(style)` helper and use it in
     `clearMarkerStyle` and the clipboard strippers (`stripMarkerStyleFromLexicalJson`
     equality check, add `--marker-working` to `MARKER_STYLE_PROPERTIES`).
   - `renderer/src/styles.css`: `span[style*="--marker-working"]::after` — ~0.75em
     CSS-animated border ring (`border-top-color: transparent; border-radius: 9999px;
     animation: spin 0.8s linear infinite; content: ''`).
   - Click-to-open: `editor.registerRootListener` click handler; target span whose
     style contains `--marker-working`; hit-test the `::after` zone (click X past the
     text `Range`'s right edge) → extract id from the bracket → `openJobFollow(id)`
     (Part 4) + `preventDefault()`. Clicks on text keep normal caret behavior.

## Part 4 — TopBar spinner + follow-chat popover

1. **New `renderer/src/app/job-events.ts`**: shared `EventTarget`;
   `openJobFollow(markerId)` / `onOpenJobFollow(cb)` — a ~15-line bus so the editor
   plugin can open the TopBar popover without threading props App→Shell→Editor.

2. **`TopBar.tsx`**:
   - Visible spinner: replace the spinning `◌` glyph with a border-ring spinner
     (`h-3 w-3 animate-spin rounded-full border-2 border-white/40 border-t-white`);
     keep `?` for `awaiting_input`.
   - Make every chip clickable (currently gated on `waiting`) — running jobs open the
     follow popover.
   - Subscribe to `onOpenJobFollow`: resolve marker id → job
     (`job.id === markerId || job.id.endsWith('-' + markerId)` for group job ids) →
     `setOpenJobId`. Existing `refresh` cleanup already closes when the job vanishes.

3. **New `renderer/src/app/JobFollowPopover.tsx`** (replaces `QuestionPopover`; same
   shell: `absolute right-0 top-7 z-50 w-96 rounded border … shadow-xl`):
   - Polls `window.api.indiana.transcript(job.id, nextSeq)` every 1s while mounted;
     appends events; `found === false` → "turn ended" note.
   - `max-h-72 overflow-y-auto` list, auto-scroll to bottom on append (skip when the
     user scrolled up).
   - Rendering: `agent` → `whitespace-pre-wrap text-xs` paragraphs; `tool` → dim
     one-liners (`⚙` prefix); `thought` → dim/collapsed; `question`/`answer` → labeled
     lines.
   - When `job.state === 'awaiting_input'`: render today's question form (message +
     textarea + Cancel/Decline/Send, lifted from `QuestionPopover`) as the popover
     footer, same `onAnswer` wiring; on answer the form clears and the transcript
     continues.

## Part 5 — Documentation run

Follow docs/AGENT_WRITING.md (frontmatter, bullets, no implementation dumps).

1. **New `docs/casablanca/CASABLANCA_AGENT_JOBS.md`** — the one page a fresh session
   needs: ownership (daemon owns jobs + transcript; Casablanca is a polling face);
   socket commands `jobs`/`answerjob`/`jobtranscript` (since-seq contract, `found:false`
   semantics); IPC channels + preload surface + 1s cadence + offline-is-ordinary;
   editor adoption ladder (own-echo → claim-patch → clean-adopt remount → dirty-warn)
   and why claims must never be clobbered; inline working spinner mechanism (style
   sentinel + CSS `::after`, byte-identical export); follow popover behavior; transcript
   lifetime (dies with job; raw log at `~/.indiana/dispatch/<id>.log`).
2. **`docs/indiana/IN_AUTORUN.md`** (already modified in working tree — edit in place):
   transport bullet now "logged *and* projected into an in-memory per-job transcript
   served via `jobtranscript`"; questions/answers appear in the transcript.
3. **`docs/casablanca/CASABLANCA_PRD.md`**: inventory additions — TopBar job chips with
   spinner, live marker claim in the open note, job follow chat.
4. If a daemon doc enumerates socket requests (check `docs/indiana/` during
   implementation), add `jobtranscript` there.

## Verification

Automated:
- `cargo test -p indiana-protocol`
- `cargo test -p indiana --features test-support` (existing autorun suite + new
  transcript test)
- `cd crates/casablanca && npm run typecheck && npm run test` (includes new
  `marker-claim.test.ts`)

End-to-end manual (Electron window, not localhost:5173):
1. `cargo build --features test-support` (builds `target/debug/mock-acp-agent`).
2. `export INDIANA_HOME=$(mktemp -d)`; git-init a fixture vault with one `doc.md`;
   write `$INDIANA_HOME/config.json` with the vault folder, `auto_run: true`, and
   `agent: { command: "<repo>/target/debug/mock-acp-agent", env: { MOCK_ACP_MODE:
   "question" } }` (question mode holds the job open — ideal for eyeballing).
3. Terminal 1: `INDIANA_HOME=… cargo run -- serve`. Terminal 2:
   `cd crates/casablanca && INDIANA_HOME=… npm run dev`.
4. Open `doc.md`, type a sentence, add `::fix -a test claim` on a new line, put the
   caret back at the end of the first sentence.
5. Verify (b): within ~1.5s the line becomes `::fix[<id>:working] -a test claim` in
   place — caret unmoved, Cmd+Z does not strip the claim, typing continues; animated
   ring appears after the bracket. Repeat with unsaved edits on another line (dirty
   buffer) — claim still lands, user text intact.
6. Verify (a): TopBar chip shows the visible ring, then `?` when the mock asks.
7. Verify (c): click the inline spinner (and separately the TopBar chip) → popover
   streams "mock agent working" + the question; answer → marker line disappears
   (clean-adopt), commit lands (`git -C <vault> log`), chip + popover clear.
8. Re-run with `MOCK_ACP_MODE=fail`: marker flips to `:failed` in place via the same
   patch path; spinner clears (style applies only to `:working`).

## Risks / accepted tradeoffs

- Popover goes blank when a turn finishes mid-read (transcript dies with job —
  decided; raw log remains for forensics).
- `spliceText` with caret inside the claimed marker line: caret may shift within the
  marker text. Rare and non-destructive.
- Marker line edited before the claim lands: patch no-ops → today's dirty-diverge
  behavior (pre-existing failure mode, not worsened; note in CASABLANCA_AGENT_JOBS.md).
- Style-sentinel spinner relies on `style*=` selectors; if marker styling ever moves to
  classes, port the sentinel (flag in the doc).
