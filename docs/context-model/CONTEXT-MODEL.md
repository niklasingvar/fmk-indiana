---
id: context-model
layer: meta
status: active
owner: shared
purpose: The schema of the context-model — how this tree is structured, read, written, linked, and kept alive.
upstream: []
review_by: 2026-10-05
updated: 2026-07-05
---

# CONTEXT-MODEL.md — The Schema of This Tree

# This file is the meta context: it governs form, never content. Content lives in the tree below.
# One line = one thing. If a line needs two sentences, it is two lines.
# Every agent reads this file first, every session, no exceptions. It is the only unconditional read.
# This tree is what turns isolated command executions into compound knowledge. Guard it accordingly.

## 0. THE THREE LAYERS (the shape of the whole system)

- Layer 1 — RAW: the repo's artifacts, code, and docs. Immutable ground truth for WHAT exists. The tree never mirrors it.
- Layer 2 — THE TREE: this folder. Agent-maintained, human-audited. The compressed truth for WHY and HOW-IT-SHOULD-BE.
- Layer 3 — THE SCHEMA: this file. It makes agents disciplined maintainers instead of generic chatbots.
- The tree is a compounding artifact: every loop must leave it equal or better, never worse.
- The tree is TRUSTED AT QUERY TIME: an `active` file is read as truth, not re-verified against sources.
- Trust is earned by lifecycle and lint (§4, §8), not by re-checking on every read — re-checking pays the cost twice.
- Corollary: a file that cannot be trusted must not be `active`. There is no third state.

## 1. THE STABILITY GRADIENT (conflict resolution in one rule)

- Every file belongs to exactly one layer; layers are ordered by stability, most stable first:
- L0 `meta` — this schema. Changes rarest; changing it is amending the constitution.
- L1 `purpose` — why the project exists, non-goals, vocabulary. The tree's compression of the repo's vision docs.
- L2 `architecture` — the shape, the invariants, the decisions and their tombstones.
- L3 `rules` — how artifacts must be made here, globally and per area.
- L4 `preferences` — operator taste; softer than rules, harder than hunches.
- L5 `learnings` — raw, fresh, unconsolidated insight from loops. The only layer agents write freely.
- L6 `journal` — index.md and log.md. Mechanical navigation and history; no knowledge lives here.
- THE CONFLICT RULE: when two files disagree, the lower L-number wins, always.
- A conflict is never silently accepted: it is fixed toward the winner or recorded as a promotion candidate (§7).
- Knowledge flows UP the gradient over time (learning → preference/rule → invariant → non-goal), compressing at every step.
- Repo-level vision docs (VISION.md, docs/PURPOSE.md) sit above L1: purpose/PURPOSE.md compresses them and yields to them.

## 2. FOLDER ARCHITECTURE

```
.indiana/context-model/
├── CONTEXT-MODEL.md          # L0 meta — this schema
├── index.md                  # L6 journal — routing file: one line per file, nothing more
├── log.md                    # L6 journal — append-only record of every loop that touched the tree
├── purpose/
│   ├── PURPOSE.md            # L1 — why, for whom, non-goals (compression of repo vision, yields to it)
│   └── GLOSSARY.md           # L1 — the ubiquitous language; one term, one meaning
├── architecture/
│   ├── ARCHITECTURE.md       # L2 — the shape, boundaries, invariants
│   └── DECISIONS.md          # L2 — decision log: choices, reversals, dead ends, tombstones
├── rules/
│   ├── GLOBAL.md             # L3 — cross-cutting creation rules for every artifact type
│   └── <area>/RULES.md       # L3 — one folder per artifact domain (presentations/, code/, docs/...)
├── preferences/
│   └── OPERATOR.md           # L4 — the operator's taste, named and dated
├── learnings/
│   ├── INBOX.md              # L5 — the single write-first target for every fresh insight
│   └── <area>.md             # L5 — consolidated learnings per domain, distilled from INBOX
└── _archive/                 # terminal — deprecated files, moved whole, frontmatter intact
```

- Folders map to layers 1:1; a file's directory and its `layer` field must agree (lint check).
- New `<area>` folders under rules/ and learnings/ are created only when the first real content for that area exists.
- Nothing else lives in this tree: no assets, no code, no artifacts, no prompt templates.
- Prompt templates live in `.indiana/indianas/<command>/prompt.md`; they LINK to this schema, they never copy it.
- Focus/task state lives in `.indiana/chief-of-staff/`; the tree stores knowledge, never TODOs.

## 3. FRONTMATTER CONTRACT (mandatory for every file, including this one)

```yaml
---
id: rules.presentations        # stable dot-namespaced ID; survives renames and moves; never reused
layer: rules                   # meta | purpose | architecture | rules | preferences | learnings | journal
status: draft                  # draft | active | deprecated
owner: shared                  # human | agent | shared — who may write the body
purpose: >-                    # ONE sentence: the question this file answers
upstream: [rules.global]       # IDs this file must not contradict; same-or-lower L-number only
review_by: 2026-10-01          # staleness fuse; past-due files get flagged by lint
updated: 2026-07-05            # bumped on every body edit, by whoever edits
---
```

- A file without frontmatter does not exist: agents skip it, lint flags it, it earns no trust.
- `id` is the file's identity; paths may change, ids may not; a dead id gets a tombstone in DECISIONS.md.
- `owner: human` — agents may propose diffs but never write the body (PURPOSE, GLOSSARY).
- `owner: agent` — agents write freely, humans audit (INBOX, log, index).
- `owner: shared` — agents write drafts and edits; promotion to `active` needs human approval (rules, architecture, preferences).
- `purpose` doubles as the file's line in index.md; keep the two identical (lint check).
- `review_by` defaults to +90 days for rules/preferences, +180 for purpose/architecture, none for journal.
- Legacy fields from older docs map cleanly: `approval: pending` reads as `status: draft`.

## 4. LIFECYCLE (the state machine)

- States: `draft` → `active` → `deprecated` → archived (in `_archive/`). No other states, no skipping to deletion.
- ANYONE may create a `draft` in any layer; drafting is cheap and always allowed.
- `draft → active`: automatic for `owner: agent` files; requires explicit human approval for `human` and `shared` files.
- Approval is an edit: the human flips `status` (or answers an `::approve`-class command); the log records it.
- `active` files are load-bearing: they are read as truth (§0) and edited only by their owner class.
- Every body edit bumps `updated`; an `active` file edited by an agent beyond its ownership is a lint violation.
- `active → deprecated`: requires a tombstone entry in architecture/DECISIONS.md naming the reason and the successor id (or "none").
- No `active` file may hold an `upstream` link to a `deprecated` file (lint error; fix the link first).
- `deprecated → _archive/`: lint moves the whole file after 30 days, frontmatter intact; git preserves everything else.
- Nothing in this tree is ever deleted; archive is the only exit, and git is the only true eraser.
- Past-due `review_by`: lint flags it; the agent proposes "re-confirm" (bump date) or "deprecate"; the owner decides.
- Staleness is a real failure mode: an untrusted-but-active file poisons the trust covenant for the whole tree.

## 5. LINKS AND DEPENDENCIES (the graph rules)

- Two kinds of edges exist, and they are not interchangeable:
- NORMATIVE edge — the `upstream` frontmatter list: "this file must not contradict these ids."
- REFERENCE edge — an inline markdown link in the body: "related material lives there."
- `upstream` may only point to the same or a more stable layer (lower L-number); this makes the normative graph acyclic by construction.
- EXCEPTION: `learnings` files carry `upstream: []` always — learnings are the one layer allowed to contradict the layers above.
- A learning that contradicts an active rule is not an error; it is a promotion candidate (§7) and lint surfaces it as such.
- Reference links may point anywhere: within the tree, to repo docs, to code paths, to artifacts.
- SSOT: every fact has exactly one home file; everywhere else it appears as a link, never as a copy.
- If readability truly demands restating, restate in at most one clause, attach the link, and treat the restatement as non-normative.
- Dangling links are allowed in `draft` files (they mark "write me later"); in `active` files they are lint errors.
- Every non-journal file must have its one line in index.md; a file missing from the index is invisible and lint flags it.
- A file with zero inbound reference links (index aside) is an orphan candidate: justify it or deprecate it.
- ANTI-MIRROR RULE: a file earns its place only by COMPRESSING facts scattered across many loops or sources.
- A file that mirrors what one grep or one file-read would reveal has negative value: it costs tokens twice and drifts. Deprecate on sight.
- Code is the SSOT for what code does; the tree is the SSOT for why it is that way and how new work must be shaped.

## 6. PLACEMENT — THE SSOT ROUTING TABLE

| You are holding...                                        | Its ONE home                          | Never in                     |
|-----------------------------------------------------------|---------------------------------------|------------------------------|
| Why the project exists, for whom, non-goals               | purpose/PURPOSE.md                     | rules, prompts               |
| What a term means (and does NOT mean)                     | purpose/GLOSSARY.md                    | scattered definitions        |
| The system's shape, boundaries, invariants                | architecture/ARCHITECTURE.md           | learnings, README mirrors    |
| A decision made or reverted, and why, and the trade-off   | architecture/DECISIONS.md              | log.md (log gets the event)  |
| A dead end already explored                               | architecture/DECISIONS.md              | anyone's memory              |
| How every artifact here must be made (cross-cutting)      | rules/GLOBAL.md                        | per-area files               |
| How ONE artifact type must be made                        | rules/<area>/RULES.md                  | prompt templates             |
| Operator taste ("no business jargon", "titles are questions") | preferences/OPERATOR.md            | rules, until proven repeatable |
| A fresh insight from ONE loop                             | learnings/INBOX.md                     | anywhere else, directly      |
| A confirmed, repeated insight for one domain              | learnings/<area>.md                    | INBOX forever                |
| What ran, when, against what                              | log.md                                 | DECISIONS.md                 |
| What exists in this tree                                  | index.md                               | duplicated catalogs          |
| Prompt wording for a command                              | .indiana/indianas/<cmd>/prompt.md      | this tree                    |
| A task, TODO, or attention item                           | .indiana/chief-of-staff/               | this tree                    |
| What the code does                                        | the code                               | this tree (anti-mirror)      |
| The artifact's content                                    | the artifact file                      | this tree                    |

- When placement is ambiguous, place it in learnings/INBOX.md with a `promote_to:` guess; consolidation will route it (§7).
- One fact, one file: if you find the same fact in two files, the lower-L file keeps it and the other gets a link. Same loop, no deferral.

## 7. WRITE-BACK PROTOCOL (how loops improve the instruction layer)

- Every command execution that touches the repo appends exactly one entry to log.md, in the greppable format:
- `## [YYYY-MM-DD] <command> | <target> — one-line outcome`
- Command classes and their write rights:
- CREATIVE (`::fix`, `::elaborate`, `::prompt`): write artifacts; in the tree, write the log entry only.
- DIAGNOSTIC (`::hate`, `::love`, `::note`): additionally write ONE atomic entry to learnings/INBOX.md.
- MAINTENANCE (`::lint`, consolidation): the only class that restructures the tree itself.
- An INBOX entry is atomic and self-contained: date, source command, the insight in ≤3 lines, a link to the artifact hunk, a `promote_to:` guess.
- `::hate` writes the DIAGNOSIS ("operator hates X because Y"), never just the symptom ("changed X").
- THE PROMOTION PIPELINE, in order, compressing at every step:
- INBOX entry → (seen once) stays; → (pattern confirmed, ~2-3 occurrences) merges into learnings/<area>.md;
- learnings/<area>.md entry → (stable, operator-confirmed) promotes to preferences/OPERATOR.md or rules/<area>/RULES.md as a draft line;
- a rule line → (violating it would be catastrophic, not just wrong) promotes to an invariant in architecture/ARCHITECTURE.md;
- an invariant → (it defines what the project IS) promotes to purpose/PURPOSE.md — human-only territory.
- Promotion MOVES knowledge; the source entry is deleted and replaced by a link to the new home. Never fork it.
- Agents promote INTO drafts; humans ratify drafts into `active` (per §4 ownership). Feedback given once is never given twice — that is the whole point.
- Any edit that adds, renames, re-statuses, or archives a file updates index.md in the same loop.
- Never write to the tree what the diff already says; write what the diff CANNOT say — the why, the taste, the trap.

## 8. READ PROTOCOL (how loops consume the tree — token budget is law)

- Reads are routed, never scanned: no command may bulk-read this tree.
- The fixed reads, every execution: (1) this file, (2) index.md, (3) purpose/PURPOSE.md.
- Then: pick at most FIVE more files from the index by relevance to the command's target; follow their `upstream` ids if unread.
- Diagnostic commands additionally read preferences/OPERATOR.md and the target area's learnings file.
- The index line must be enough to decide relevance; if it isn't, the index line is broken — fix it in the same loop.
- Read `active` files as truth (§0); read `draft` files as proposals; never read `deprecated` files for guidance.
- If the budget is insufficient for the task, say so in the loop output; never silently skim the whole tree instead.

## 9. LINT (the health pass — not optional, drift is the killer)

- Lint runs via `::lint`, and at minimum every 10 loops or 14 days, whichever first.
- Checks, in order:
- Frontmatter: present, schema-valid, layer/directory agreement, id uniqueness, purpose==index-line.
- Lifecycle: past-due `review_by`; `deprecated` files past 30 days (move to _archive/); active-upstream-to-deprecated edges.
- Graph: dangling links in active files; orphans; files missing from index; normative edges pointing down-gradient.
- SSOT: the same fact stated in two files; mirror files that a grep would replace (anti-mirror sweep).
- Contradictions: learnings that conflict with active rules → surface as promotion candidates, not errors.
- INBOX pressure: more than 15 entries, or any entry older than 14 days → consolidation is due NOW.
- Budgets: any file over 120 lines (split or compress); tree over 25 active files (consolidate).
- Lint NEVER deletes and never rewrites `human`-owned bodies; it fixes mechanics, drafts proposals, and reports the rest.
- The lint report is a log entry plus draft fixes; the human ratifies anything normative.

## 10. BUDGETS (compression is the product)

- The tree competes for the same context window as the actual work; every line here taxes every loop.
- Hard ceilings: ≤120 lines per file, ≤25 active files, ≤15 INBOX entries, ≤5 routed reads per loop, 1 index line per file.
- When a ceiling is hit the answer is always compression, never a ceiling raise.
- Growth pattern: start with the seed files only; a new area file is born from real INBOX pressure, never from anticipation.
- Prefer editing an existing file over creating a new one; new files need a reason the index line can state.
- Distillation must never silently drop a learning or a tombstone: compress the wording, keep the trap.

## 11. THE COVENANT

- The loops are the memory: chat is stateless, this tree is not, and that difference is the product's soul.
- Every loop leaves the tree equal or better — the artifact improves AND the system's understanding of the project improves.
- Trust is the currency: an active file is believed, so keeping active files true outranks adding new ones.
- Compress relentlessly, link instead of copy, promote instead of repeat, and never mirror what grep already knows.
