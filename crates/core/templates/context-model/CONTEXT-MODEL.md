---
id: context-model
layer: meta
status: active
owner: shared
purpose: The schema of the context-model — how this tree is structured, read, written, and kept alive.
upstream: []
review_by: 2026-10-05
updated: 2026-07-05
---

# CONTEXT-MODEL.md — The Schema of This Tree

# This file governs form, never content. Content lives in the tree below.
# One line = one thing. Every agent reads this file first, every session — the only unconditional read.
# This tree turns isolated command executions into compound knowledge. Guard it accordingly.

## 0. THE THREE LAYERS

- Layer 1 — RAW: the repo's artifacts, code, and docs. Ground truth for WHAT exists. The tree never mirrors it.
- Layer 2 — THE TREE: this folder. Agent-maintained, human-audited. The compressed truth for WHY and HOW-IT-SHOULD-BE.
- Layer 3 — THE SCHEMA: this file.
- The tree is TRUSTED AT QUERY TIME: an `active` file is read as truth, not re-verified against sources.
- A file that cannot be trusted must not be `active`. Every loop leaves the tree equal or better.

## 1. THE STABILITY GRADIENT

- Layers ordered by stability, most stable first:
- L0 `meta` — this schema. L1 `purpose` — why, non-goals, vocabulary. L2 `architecture` — shape, invariants, decisions.
- L3 `rules` — how artifacts must be made here. L4 `preferences` — operator taste. L5 `learnings` — fresh, unconsolidated insight.
- L6 `journal` — index.md and log.md. Mechanical navigation and history; no knowledge lives here.
- THE CONFLICT RULE: when two files disagree, the lower L-number wins, always.
- Knowledge flows UP the gradient over time (learning → preference/rule → invariant → non-goal), compressing at every step.
- Repo-level vision docs sit above L1: purpose/PURPOSE.md compresses them and yields to them.

## 2. FOLDER ARCHITECTURE

```
.indiana/context-model/
├── CONTEXT-MODEL.md          # L0 meta — this schema
├── index.md                  # L6 journal — routing file: one line per file
├── log.md                    # L6 journal — append-only record of every loop
├── purpose/PURPOSE.md        # L1 — why, for whom, non-goals
├── architecture/             # L2 — ARCHITECTURE.md, DECISIONS.md (born from real content)
├── rules/                    # L3 — GLOBAL.md, <area>/RULES.md (born from real content)
├── preferences/              # L4 — OPERATOR.md (born from real content)
├── learnings/
│   └── INBOX.md              # L5 — the single write-first target for every fresh insight
└── _archive/                 # terminal — deprecated files, moved whole
```

- New folders and files are created only when the first real content for them exists — never from anticipation.
- Nothing else lives in this tree: no assets, no code, no artifacts, no prompt templates.
- Prompt templates live in `.indiana/indianas/<command>/prompt.md`; they LINK to this schema, they never copy it.
- Focus/task state lives in `.indiana/chief-of-staff/`; the tree stores knowledge, never TODOs.

## 3. FRONTMATTER CONTRACT (mandatory for every file)

- Fields: `id` (stable, dot-namespaced), `layer`, `status` (draft|active|deprecated), `owner` (human|agent|shared),
- `purpose` (one sentence; identical to the file's index.md line), `upstream` (ids this file must not contradict),
- `review_by` (staleness fuse), `updated` (bumped on every body edit).
- A file without frontmatter does not exist: agents skip it and it earns no trust.
- `owner: human` — agents propose diffs, never write the body. `owner: agent` — agents write freely, humans audit.
- `owner: shared` — agents write drafts; promotion to `active` needs human approval.

## 4. LIFECYCLE

- `draft` → `active` → `deprecated` → archived (in `_archive/`). No other states, nothing is ever deleted.
- Anyone may create a `draft`. `draft → active` is automatic for `owner: agent`; needs human approval otherwise.
- `active → deprecated` requires a tombstone in architecture/DECISIONS.md naming reason and successor.

## 5. LINKS AND SSOT

- `upstream` points only to the same or a more stable layer; `learnings` always carry `upstream: []`.
- A learning that contradicts an active rule is not an error; it is a promotion candidate.
- SSOT: every fact has exactly one home file; everywhere else it appears as a link, never a copy.
- ANTI-MIRROR RULE: a file earns its place only by COMPRESSING many loops or sources. A file one grep would replace has negative value.
- Code is the SSOT for what code does; the tree is the SSOT for why, and how new work must be shaped.

## 6. WRITE-BACK PROTOCOL (every loop)

- Every command execution appends one entry to log.md: `## [YYYY-MM-DD] <command> | <target> — one-line outcome`
- CREATIVE commands (`::fix`, `::elaborate`, `::prompt`): write artifacts; in the tree, the log entry only.
- DIAGNOSTIC commands (`::hate`, `::love`, `::note`): additionally write ONE atomic entry to learnings/INBOX.md —
- date, source command, the insight in ≤3 lines, a link to the artifact, a `promote_to:` guess.
- `::hate` writes the DIAGNOSIS ("operator hates X because Y"), never just the symptom.
- Promotion pipeline: INBOX → (pattern confirmed) learnings/<area>.md → (operator-confirmed) preferences or rules →
- (catastrophic-if-violated) architecture invariant. Promotion MOVES knowledge; the source is replaced by a link.
- Agents promote INTO drafts; humans ratify. Feedback given once is never given twice — that is the whole point.
- Any edit that adds, renames, re-statuses, or archives a file updates index.md in the same loop.
- Never write to the tree what the diff already says; write what the diff CANNOT say — the why, the taste, the trap.

## 7. READ PROTOCOL (token budget is law)

- Reads are routed, never scanned: no command may bulk-read this tree.
- The fixed reads, every execution: (1) this file, (2) index.md, (3) purpose/PURPOSE.md.
- Then: pick at most FIVE more files from the index by relevance to the command's target.
- Diagnostic commands additionally read preferences/OPERATOR.md and the target area's learnings file, when they exist.
- Read `active` files as truth; read `draft` files as proposals; never read `deprecated` files for guidance.

## 8. BUDGETS

- Hard ceilings: ≤120 lines per file, ≤25 active files, ≤15 INBOX entries, ≤5 routed reads per loop, 1 index line per file.
- When a ceiling is hit the answer is always compression, never a ceiling raise.
- Prefer editing an existing file over creating a new one.

## 9. THE COVENANT

- The loops are the memory: chat is stateless, this tree is not, and that difference is the product's soul.
- Compress relentlessly, link instead of copy, promote instead of repeat, and never mirror what grep already knows.
