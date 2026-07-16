---
status: draft
purpose: The fundamentals in four tiers — universal beliefs, app beliefs, structural laws, loop practices. Each row is definition, rule, test. Narrative lives in VISION.md.
approval: pending
---

# Fundamentals
*Universal beliefs. Hold everywhere, app or not.*

SINGLE SOURCE OF TRUTH
- Definition: every fact has exactly one home file; every other appearance is a link, never a copy.
- Rule: to change a fact, edit its home; readability may restate in at most one clause, with the link attached.
- Test: the same fact found in two files is a violation — the more stable file keeps it, the other links.

ELEPHANT PRINCIPLE
- Definition: anything too big to hold is cut into pieces small enough to finish and review in one pass.
- Rule: small files, small problems, small loops; every file carries a max-rows ceiling.
- Rule: when a ceiling is hit, the answer is compression or a split — never a ceiling raise.
- Test: a file or task that cannot be reviewed in one sitting is too big.

# App Fundamentals
*What the app believes. Change one and it is a different product.*

CONTENT OVER CHAT
- Definition: the artifact is the interface; work and feedback happen in the file.
- Rule: feedback is written into the artifact as `::` markers, never described in a conversation.
- Test: if giving feedback requires explaining where in a chat window, this is broken.

FOLDER IS THE UNIT OF WORK
- Definition: a folder is a mission — artifact, context, and configuration travel together.
- Rule: everything an agent needs lives inside the folder (`.indiana/`); no state outside it.
- Test: point any agent at the folder and it has everything; move the folder and nothing is lost.

KNOWLEDGE COMPOUNDS THROUGH LOOPS
- Definition: the loops are the memory; chat is stateless, the tree is not.
- Rule: every loop leaves the tree equal or better — the artifact improves and so does the system's understanding.
- Test: feedback given once is never given twice.

DOMAIN ARCHITECTURE > TECH
- Definition: the tree is shaped by what the project is about, never by the technology it happens to use.
- Rule: tech is replaceable and conforms to the domain model; the tree is never reshaped for a framework.
- Test: swap the stack and no folder in the tree is renamed.

HARNESS AGNOSTIC
- Definition: the system never runs its own agent; compiled markers are handed to the harness the user already has.
- Rule: their tokens, their quota, their harness — never own the token bill.
- Test: swap Claude Code for Codex or Cursor and nothing in the folder changes.

# Principles
*How knowledge is structured: space, time, relations. Change one and the tree is restructured.*

CONE-SHAPED TREE ARCHITECTURE
- Definition (space): every file sits on a stability gradient — purpose, architecture, rules, preferences, learnings — most stable at the top.
- Rule: knowledge flows up the gradient over time, compressing at every step — wide base, narrow top.
- Rule: when two files disagree, the more stable layer wins, always.
- Test: the top layers are few and short; a fat top or a contradiction resolved downward breaks the cone.

FILE LIFE CYCLE MANAGEMENT
- Definition (time): every file is in exactly one state — draft → active → deprecated → archived; no other states, no skipping.
- Rule: active means trusted at query time — read as truth, never re-verified against sources.
- Rule: nothing is deleted; archive is the only exit, git the only true eraser.
- Test: a file that cannot be trusted must not be active — there is no third state.

FULL DEPENDENCY MANAGEMENT
- Definition (relations): every dependency is declared — normative upstream edges in frontmatter, reference links in the body.
- Rule: upstream edges point only to same-or-more-stable layers; the normative graph is acyclic by construction.
- Rule: if a file must not contradict another, that edge is written down — never implied.
- Test: dangling links in active files and undeclared contradictions fail lint; an orphan justifies itself or is deprecated.

# Execution
*What every loop does. Change one and only prompts and lint change.*

FRONTMATTER ON EVERY FILE
- Definition: YAML frontmatter is the machine-readable contract — id, layer, status, owner, purpose, upstream, review date.
- Rule: a file without frontmatter does not exist — agents skip it, lint flags it, it earns no trust.
- Test: lifecycle and dependencies are enforceable from frontmatter alone; what is not in frontmatter is not enforced.

DOCUMENT WHY, NOT WHAT
- Definition: docs record intent, trade-offs, and traps — what the diff cannot say.
- Rule: never mirror what a grep or file-read already reveals; code is the truth for what code does.
- Test: a file that restates one grep has negative value — deprecate on sight.

PROMOTE, NEVER FORK
- Definition: knowledge moves up the cone as it proves out — inbox → learning → rule → invariant → purpose.
- Rule: promotion moves the fact; the source is deleted and replaced by a link, never copied.
- Test: the same insight alive in two layers means a promotion forked — merge and link.

MARKDOWN AS CODE
- Definition: markdown is treated as source code — parseable, lintable, diffable.
- Rule: one line = one thing; a line that needs two sentences is two lines.
- Test: if lint cannot check it, restructure it until lint can.
