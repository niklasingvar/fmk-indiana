---
status: draft
purpose: Index of the fundamentals in four tiers — universal beliefs, app beliefs, structural laws, loop practices. One file per fundamental in fundamentals/. Narrative lives in VISION.md.
approval: pending
---

# Fundamentals
*Universal beliefs. Hold everywhere, app or not.*

[SINGLE SOURCE OF TRUTH](fundamentals/SINGLE_SOURCE_OF_TRUTH.md)
- Every fact has exactly one home; everywhere else it is a link, never a copy.

[ELEPHANT PRINCIPLE](fundamentals/ELEPHANT_PRINCIPLE.md)
- Eat it one bite at a time: small files, small problems, small loops; ceilings are parameters, not prose.

# App Fundamentals
*What the app believes. Change one and it is a different product.*

## [CONTENT OVER CHAT](fundamentals/CONTENT_OVER_CHAT.md)
- The artifact is the interface; feedback lives in the file as `::` markers, not in a conversation.

[FOLDER IS THE UNIT OF WORK](fundamentals/FOLDER_IS_THE_UNIT_OF_WORK.md)
- A folder is a mission: artifact, context, and configuration travel together; no state outside it.

[KNOWLEDGE COMPOUNDS THROUGH LOOPS](fundamentals/KNOWLEDGE_COMPOUNDS_THROUGH_LOOPS.md)
- The loops are the memory; feedback given once is never given twice.

[DOMAIN ARCHITECTURE > TECH](fundamentals/DOMAIN_ARCHITECTURE_OVER_TECH.md)
- The tree is shaped by what the project is about, never by the technology it happens to use.

[HARNESS AGNOSTIC](fundamentals/HARNESS_AGNOSTIC.md)
- Never run an own agent or own the token bill; any harness connects to the folder.

# Principles
*How knowledge is structured: space, time, relations. Change one and the tree is restructured.*

[CONE-SHAPED TREE ARCHITECTURE](fundamentals/CONE_SHAPED_TREE_ARCHITECTURE.md)
- Space: knowledge flows up a stability gradient, compressing at every step; the more stable layer always wins.

[FILE LIFE CYCLE MANAGEMENT](fundamentals/FILE_LIFE_CYCLE_MANAGEMENT.md)
- Time: draft → active → deprecated → archived; active is trusted at query time, nothing is deleted.

[FULL DEPENDENCY MANAGEMENT](fundamentals/FULL_DEPENDENCY_MANAGEMENT.md)
- Relations: every dependency is a declared upstream edge pointing to a more stable layer — acyclic by construction.

# Execution
*What every loop does. Change one and only prompts and lint change.*

[FRONTMATTER ON EVERY FILE](fundamentals/FRONTMATTER_ON_EVERY_FILE.md)
- The contract carrier: id, layer, status, owner, upstream. No frontmatter, no trust.

[DOCUMENT WHY, NOT WHAT](fundamentals/DOCUMENT_WHY_NOT_WHAT.md)
- Write what the diff cannot say; never mirror what a grep already knows.

[PROMOTE, NEVER FORK](fundamentals/PROMOTE_NEVER_FORK.md)
- Knowledge moves up the cone; the source becomes a link, never a duplicate.

[MARKDOWN AS CODE](fundamentals/MARKDOWN_AS_CODE.md)
- One line = one thing; structure carries the meaning — body text should almost never need human reading.

# Incorporation
*How the fundamentals reach beyond this file.*

- This repo: [CLAUDE.md](CLAUDE.md) puts this index first in the floor plan; each fundamental's file names its enforcement point.
- Future AI (system prompt): the embedded templates in `crates/core/templates/` — the loop preamble and the context-model seed — carry the fundamentals into every monitored repo; every agent loop reads them before acting.
- Parameters: fundamentals name tunable ceilings; values live in `.indiana/casablanca/settings.json` per repo (today: `maxRowsPerFile`).
- Rule of thumb: a fundamental that reaches neither a template, a lint check, nor a settings key is decoration — wire it or cut it.
