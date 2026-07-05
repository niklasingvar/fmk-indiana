---
status: draft
purpose: Specify the context-model — per-repo memory that makes knowledge compound through loops.
approval: pending
---

# CM — PRD

> The memory layer of [VISION.md](../../VISION.md). Lives at `.indiana/context-model/` in every monitored repo ([IN_FOLDER.md](../indiana/IN_FOLDER.md)).

## What it is
- The project's brain: rules for how things should be created in this repo, plus the knowledge accumulated from every loop that has run.
- Plain markdown files — agent-readable, git-versionable, human-editable.
- Organized as a strict hierarchical information tree; depth 1 is domain modelling.

## The contract (bidirectional)
- Read: every compiled command prompt directs the agent into the context-model before acting.
- Write: feedback commands (`::hate`, `::love`) instruct the agent to write the extracted rule back.
- This is what turns isolated command executions into compound knowledge. Feedback given once is never given twice.

## Method
- The loop is an ADLC (https://www.voodootikigod.com/series/adlc) improved with one extra step: every cycle updates the context-model itself. The project gets smarter, not just bigger.

## Status
- MVP wired (2026-07). The schema is [files/CONTEXT-MODEL.md](../../files/CONTEXT-MODEL.md), compressed into the shipped seed
  `crates/core/templates/context-model/CONTEXT-MODEL.md`; scaffold seeds the schema, `index.md`, `log.md`,
  `purpose/PURPOSE.md`, and `learnings/INBOX.md`.
- Read/write contract wired: every rendered copy payload opens with a loop preamble
  (`crates/core/templates/preamble.md`, embedded in `render_text`) — read protocol in, log entry and
  `.indiana/chief-of-staff/focus.md` todo update out. `::hate`/`::love`/`::note` templates instruct the
  `learnings/INBOX.md` write-back. Remaining Phase 5 work: [ACTION_PLAN.md](../../ACTION_PLAN.md).

## Rules
- One row = one thing.
- Every file has frontmatter.
- Single source of truth; link, do not duplicate.
- Per-repo only; no global memory yet ([VISION.md](../../VISION.md) non-goals).
