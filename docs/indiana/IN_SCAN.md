---
purpose: Specify the scan engine — how Indiana finds indianas, fast, without owning the truth.
max_lines: 70
status: draft
approval: pending
---

# IN_SCAN — scan engine

> How Indiana finds indianas. Markers: [IN_COMMANDS.md](IN_COMMANDS.md). Scope: [IN_SCOPE.md](IN_SCOPE.md). IDs: [IN_IDENTITY.md](IN_IDENTITY.md).

## Stance
- Markdown is the source of truth. The index is a throwaway view, rebuilt each scan.
- Read-only by default. The only write is a one-time ID injection ([IN_IDENTITY.md](IN_IDENTITY.md)).
- Cheap enough to run on every keystroke-debounced save.
- Any scan (not just the daemon) may inject IDs ([IN_IDENTITY.md](IN_IDENTITY.md)). A `--read-only` flag suppresses writes.

## Find
- An indiana sits at column 0, or inline at end of a content line.
- Why column 0 / `::`: survives every markdown parser, `rg '^::'` has zero false positives.
- Line-oriented, stateless per line. No cross-line state except fence tracking.
- One indiana per line. A line bearing two or more markers is ambiguous — skip it, warn, leave bytes untouched.

## Code fences and spans
- Track ` ``` ` and `~~~` fences independently; one does not close the other. Two states, not one boolean.
- An indiana inside an open fence is ignored — it is sample text, not a command.
- Inline code spans suppress markers too, by the same rule: a `::` inside a backtick span is sample text. CommonMark run matching — an opener of N backticks closes only at the next run of exactly N — so a span may itself contain a `` ``` `` shown inline. An unmatched backtick is literal; a marker after it still counts.
- Why: agents quote `::` in code, fenced or inline; those must never trigger. Found by dogfooding — Indiana's own specs quote every marker inline, and a fence-only rule reported dozens of false positives.
- An unclosed fence swallows the rest of the file (everything after is ignored). Accepted; warn on EOF inside an open fence so the silence is visible.
- YAML frontmatter: a leading `---` block at file start (line 1, closed by the next `---`) is ignored except for explicit column-zero property comments shaped `# frontmatter.<key> ::<cmd>`. Values, ordinary comments, and indented block-scalar content stay inert. This is the only `---` special case — a thematic break mid-document never starts one.
- Indented code blocks are not detected. A `::` inside a 4-space block still resolves by the column-0 / end-of-line rule; we do not track paragraph state to exclude it. Simplicity over completeness.

## Walk and watch
- Startup: full walk of all markdown under the repo root.
- Steady state: event-driven via FSEvents; ~300 ms debounce after the last change.
- The debounce worker blocks on the event channel while idle; no polling timer wakes merely to check for work.
- FSEvents coalesces bursts — `git checkout`, bulk `sed`, editor auto-save fire hundreds of events in milliseconds.
- Debounce the event stream globally, then rebuild the shared index once after the tree goes quiet.
- Exclude `.indiana/` from the walk — Indiana's own scratch is not content.
- Why event-driven: near-zero idle CPU; the human feels instant pickup.

## Own writes
- After Indiana injects an ID, suppress that path's events for ~500 ms.
- Why: the write Indiana just made must not re-trigger a scan of itself.
- Risk: a real human edit landing inside that window is suppressed for one cycle. Accepted; it surfaces on the next event.

## Concurrency and races
- The mtime guard ([IN_IDENTITY.md](IN_IDENTITY.md)) narrows the inject race; it does not close it.
- Rename is atomic on APFS, but the file can change between the mtime check and the rename — a concurrent human edit can still be overwritten.
- Mitigation: on a guard trip, ID injection retries from a fresh scan of that one file, not the whole repo.
- Worst case: a marker misses its identity for a single scan cycle, then gets it on the next.

## Targets
- Pickup latency under ~500 ms from save to updated index.
- Near-zero CPU at idle. Single static `aarch64-apple-darwin` binary, no runtime deps.
- Recoverable: delete the index, rescan, full state returns from source.

## Decided
- Full walk on startup, event-driven after. Simple and recoverable.
- Incremental startup walk deferred until a measured repo breaks the latency budget — not before.
