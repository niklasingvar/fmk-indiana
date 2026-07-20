---
status: draft
purpose: One page for the agent-job surface — daemon jobs/transcript over the socket, editor claim adoption, spinners, follow popover.
approval: pending
---

# CASABLANCA_AGENT_JOBS — following daemon agent turns

> Job lifecycle and dispatch: [../indiana/IN_AUTORUN.md](../indiana/IN_AUTORUN.md). Full code ownership map: [../indiana/IN_AUTORUN_ARCHITECTURE.md](../indiana/IN_AUTORUN_ARCHITECTURE.md). Daemon socket: [../indiana/IN_DAEMON.md](../indiana/IN_DAEMON.md). Feature inventory: [CASABLANCA_PRD.md](CASABLANCA_PRD.md).

## Ownership
- The daemon owns jobs and their transcripts; both are daemon memory and die with the turn (restart or resolution). The raw ACP log at `~/.indiana/dispatch/<id>.log` is the durable record.
- Casablanca is a polling face: it renders projections and forwards answers, never holds job state of its own.
- An offline daemon is an ordinary state, not an error — jobs read as none, transcripts as ended.

## Socket commands used
- `jobs` → live turns `{ id, root, markers, state: running|awaiting_input, question }`.
- `answerjob { job_id, action, answer }` → resumes the one pending form question.
- `jobtranscript { job_id, since_seq }` → `{ found, events, next_seq }`; events are `{ seq, kind: agent|thought|tool|question|answer, text }`.
- Transcript contract: `seq` is monotonic per job; poll from the last seen event's `seq`, because streamed chunks merge into the tail event (its seq stays, its text grows). `found: false` means the turn ended.
- Wire types: `crates/indiana-protocol/src/lib.rs`. Projection: `crates/indiana/src/dispatch.rs` (`JobRegistry`).

## Casablanca plumbing
- Socket client `main/lib/indiana.ts` → IPC invoke channels `indiana:jobs`, `indiana:answer-job`, `indiana:job-transcript` (`shared/ipc.ts`, `main/ipc.ts`) → preload `window.api.indiana` (`preload/index.ts`).
- `useAgentJobs` polls `jobs` at 1s; an open follow popover polls `jobtranscript` at 1s (`renderer/src/app/agents/useAgentJobs.ts`, `renderer/src/app/agents/JobFollowPopover.tsx`). Compact indicators render left of the stage panel icons (`AgentIndicators.tsx` via `TopBar.tsx`).
- Batch dispatch is never invisible (`GroupButtons.tsx`): on accept the batch button spins immediately (`launching` bridges the gap until the 1s jobs poll sees the turn, with an 8s grace for turns that end between polls), the button stays a spinner (or `?` while awaiting input) for the whole turn and opens the follow popover on click (`shared/batch-job.ts` maps `group-N-`/`agent-name-` job-id prefixes to buttons), rejection and turn-end show a visible toast under the button — not just a tooltip.

## Editor adoption ladder
- `note:changed` (main watcher push) re-reads the open note; `useVault.ts` picks the first matching branch:
  - own autosave echo → ignore.
  - disk equals live draft → re-baseline only.
  - claim patch: the only change is marker lines gaining/changing an `[id:status]` bracket (`shared/marker-claim.ts`) → splice into the live editor, no remount. Works on dirty buffers; cursor, undo, and unsaved edits survive.
  - clean buffer → adopt wholesale via remount (`noteVersion` bump).
  - dirty and diverged → keep user text, warn; next autosave wins.
- The claim must never be clobbered: a stale buffer autosaving over `[id:working]` makes the daemon see a fresh marker and dispatch it again (duplicate turns and commits). The claim-patch branch is the defense — it lands the bracket in the buffer before the next autosave.
- The splice is history-merged (`renderer/src/editor/plugins/MarkerClaimPlugin.tsx`): undo must not strip a claim, or the unclaimed marker would be saved back and re-dispatched.
- A claim whose line was edited away in the unsaved buffer no-ops and degrades to the dirty-diverge path.

## Inline working spinner
- `MarkerHighlightPlugin.tsx` gives the `[id:working]` bracket a style variant carrying the `--marker-working` custom property; `styles.css` attaches an animated ::after ring to `span[style*="--marker-working"]`.
- Pure presentation: no document text, byte-identical markdown export, bracket stays editable. If marker styling ever moves from inline styles to classes, port the sentinel with it.
- Clicking the ring (hit-tested past the text's right edge) opens the follow popover via the `app/agents/job-events.ts` bus; clicking the text keeps caret behavior.

## Follow popover
- Anchored to a TopBar agent indicator; also opened by the inline spinner (marker id → job id, group jobs match by `group-N-<id>` suffix).
- Renders the transcript kinds distinctly; while `awaiting_input` the one-string question form is the footer, and the transcript continues after answering.
- Shows "turn ended" when `found: false`; there is deliberately no post-turn transcript retention.
