# CLAUDE.md — The Meta-Model of This Project
# This file is normative. It outranks convention, habit, and fashion. It does not outrank the user.
# One line = one thing. If a line needs two sentences, it is two lines.
# This is intent and direction, not a replica of the code. The code says WHAT; this file says WHY and NEVER.
# If code and this file disagree, one of them is a bug. Stop and surface the conflict before proceeding.
# Read this file fully before your first edit in this repo. Skimming it is how trendslop gets in.

## 0. WHAT THIS FILE IS

- This is the project's long-term memory: learnings, caveats, invariants, and dead ends already explored.
- It exists to prevent trendslop: the silent replacement of deliberate decisions with whatever is currently fashionable.
- It exists so that context is never re-derived from scratch, badly, by every new agent session.
- It captures the depth that git history hides and code comments cannot hold.
- Every line here was paid for — with a bug, an outage, a rewrite, or a week of confusion.
- Deleting a line here without recording why is destroying institutional memory.
- This file is read by agents; write for a reader with zero session memory and full repo access.
- Nothing in this file describes what the code already states plainly. Redundancy here is rot.

## 1. PRIME DIRECTIVES

- Understand intent before touching implementation.
- Preserve invariants over preserving code; code is replaceable, invariants are not.
- Prefer the boring solution that will still be correct in five years.
- Never introduce a technology, pattern, or abstraction because it is popular. Introduce it because a stated problem demands it.
- The absence of a feature can be a feature. Check §3 (Non-Goals) before adding anything.
- Small, reversible changes beat large, impressive ones.
- If you cannot explain a change in one sentence of intent, do not make it.
- When in doubt, ask the maintainer; when the maintainer is absent, do the conservative thing.
- Leave the repo more legible than you found it, or at minimum not less.
- You are a steward here, not an author. Authors express themselves; stewards protect intent.

## 2. PROJECT IDENTITY — WHY THIS EXISTS

- Name: {{PROJECT_NAME}}.
- One-sentence purpose: {{what this project does, for whom, and why it must exist}}.
- The single problem this project solves better than alternatives: {{the differentiator}}.
- The user we optimize for: {{primary user persona}} — not the hypothetical power user, not the demo audience.
- The core promise to that user: {{the one guarantee we never break}}.
- Success is measured by: {{the real metric}}, not by lines of code, stars, or architectural elegance.
- The project's lifespan expectation: {{e.g., 10+ years of maintenance by a small team}}.
- The maintenance reality: assume the future maintainer is tired, busy, and reading this at 2 a.m. during an incident.
- Everything in this repo should be explicable to that maintainer in under a minute.

## 3. NON-GOALS — WHAT WE DELIBERATELY DO NOT DO

- Non-goals are decisions, not omissions. Each one below was argued and settled.
- We do not chase feature parity with {{competitor/reference project}}.
- We do not support {{explicitly excluded platform/use case}}; supporting it would compromise {{the core promise}}.
- We do not build for hypothetical scale; we build for {{actual measured scale}} with headroom, not fantasy.
- We do not add configuration options to avoid making a decision. A config flag is a decision deferred onto the user.
- We do not expose internals as public API "just in case."
- We do not accept complexity today for flexibility we cannot name a concrete use for.
- We do not internationalize / theme / pluginize until a real, named demand exists ({{status}}).
- We do not rewrite working subsystems for stylistic reasons. Ever.
- Re-opening a non-goal requires a written argument in §19, not a pull request.

## 4. INVARIANTS — NEVER BREAK, NO EXCEPTIONS

- Invariants are load-bearing. Violating one is a critical bug even if all tests pass.
- INV-1: {{e.g., The public API never breaks without a major version bump and a migration path}}.
- INV-2: {{e.g., User data is never mutated in place; every write goes through the journal}}.
- INV-3: {{e.g., The core path has zero network calls; it must work fully offline}}.
- INV-4: {{e.g., IDs are opaque and immutable once issued; they encode nothing}}.
- INV-5: {{e.g., All timestamps are UTC in storage; timezone is a presentation concern only}}.
- INV-6: {{e.g., The system is deterministic given the same input and seed}}.
- INV-7: {{e.g., No dependency in the core module may pull in transitive dependencies of its own}}.
- INV-8: {{e.g., Every state transition is representable as a plain, serializable value}}.
- Every invariant must have at least one test that fails loudly when it breaks.
- If a task seems to require breaking an invariant, the task is wrong or the invariant list is stale — halt and escalate.
- New invariants are added here the moment they are discovered, in the same commit that relies on them.

## 5. ARCHITECTURE — THE SHAPE AND ITS REASONS

- The architecture in one line: {{e.g., a pure core, an imperative shell, and adapters at every boundary}}.
- The dependency direction is one-way: {{outer layers}} depend on {{inner layers}}, never the reverse.
- The core is deliberately dependency-light because every dependency there taxes every user forever.
- Boundaries are drawn along rate-of-change, not along technology: things that change together live together.
- Module {{X}} exists to isolate {{volatile concern}}; do not let it leak into {{stable concern}}.
- We chose {{pattern A}} over {{pattern B}} because {{concrete reason, e.g., B failed under concurrent writes in 2024}}.
- The database schema is the most expensive thing to change; treat schema edits as constitutional amendments.
- Synchronous by default; async only where a measured wait justifies it. Async-everywhere was tried and reverted ({{ref}}).
- There is exactly one source of truth for {{critical state}}: {{location}}. Everything else is a cache and may be deleted.
- Caches must be safe to wipe at any moment; if wiping a cache loses data, it was never a cache.
- The plugin/extension boundary is {{interface}}; nothing crosses it except {{allowed types}}.
- Cross-cutting concerns (logging, auth, metrics) enter through {{mechanism}}, never inline in business logic.
- If a new module doesn't fit the shape, question the module before questioning the shape.
- The shape may be wrong; changing it requires a §19 entry, not a refactor commit.

### 5.1 Legibility within the shape

- Code is read hundreds of times per write; optimize for the reader by default.
- A function should be explicable by its name plus signature; if it needs a paragraph, split it.
- Depth of nesting is a complexity alarm; past three levels, extract or invert.
- Cleverness is a cost; the reviewer at 2 a.m. (§2) is the audience, not the language lawyer.
- Match the surrounding style even where you dislike it; local consistency beats global preference.
- The style is whatever the formatter and linter enforce; style debates outside their configs are noise.
- Dead code is deleted, not commented out; git remembers so the file doesn't have to.
- One concept per file where the language allows; grep-ability is a design goal.
- Make illegal states unrepresentable in types before guarding them with runtime checks.
- If you must be clever (hot path, §13), fence it with a comment linking the benchmark that justifies it.

## 6. ANTI-TRENDSLOP RULES

- Trendslop: code changed to match fashion rather than need. It is the primary decay vector for this repo.
- No framework migrations without a named, measured, current pain. "Old" is not pain.
- No new abstraction until the same shape has appeared three times in real code. Two is coincidence.
- No design patterns applied for their own sake; patterns are vocabulary for solutions, not solutions seeking problems.
- No microservices, event sourcing, CQRS, or distributed anything unless the load numbers in §13 demand it.
- No swapping a working library for a trendier one with the same feature set.
- No "modernizing" syntax across files you were not otherwise asked to touch.
- No speculative generality: no generics, hooks, or interfaces for callers that do not exist.
- No AI-flavored boilerplate: exhaustive comments on obvious lines, defensive checks against impossible states, or apologetic naming.
- No README-driven architecture: we do not adopt a tool because its landing page is beautiful.
- Benchmarks from blog posts are marketing; only benchmarks from this repo's harness count.
- The newest stable release is not automatically the right release; see the pinning policy in §9.
- When you feel the urge to "clean this up while I'm here" — record the urge in §21 and stay on task.
- Fashion cycles are shorter than this project's lifespan. Every trend adopted is a future migration owed.
- The correct response to "everyone does X now" is: "what problem of ours does X solve, and at what cost?"
- Deleting trendslop that already crept in is always in scope; adding more never is.
- A trend that survives five years and still solves a named problem here may apply via §19 — that is the only door in.
- Novelty budget: at most one genuinely new technique per quarter, and it must land in §20 with its reason.
- Boring is a compliment in this repo; write it in reviews and mean it.

## 7. HARD-WON LEARNINGS — DO NOT RE-LEARN THESE

- Each learning below cost real time. Treat them as pre-paid tuition.
- LRN-1: {{e.g., The upstream API silently truncates payloads over 1 MB; it returns 200 anyway. Always verify length}}.
- LRN-2: {{e.g., Retries without jitter took the service down harder than the original failure. Jitter is mandatory}}.
- LRN-3: {{e.g., The ORM's lazy loading caused N+1 queries in every list view; eager-load explicitly, always}}.
- LRN-4: {{e.g., Floating point money broke reconciliation in month 3. All money is integer minor units}}.
- LRN-5: {{e.g., Filesystem watchers fire duplicate events on macOS; debounce or suffer double-processing}}.
- LRN-6: {{e.g., The "temporary" feature flag from 2023 became load-bearing; flags need expiry dates}}.
- LRN-7: {{e.g., Parsing with regex was tried twice for {{format}}; both times ended in a real parser. Skip to the parser}}.
- LRN-8: {{e.g., Locale-dependent string comparison corrupted sort order for Turkish users; compare bytewise or with explicit collation}}.
- LRN-9: {{e.g., Background jobs must be idempotent because the queue redelivers under load. It will redeliver}}.
- LRN-10: {{e.g., Our users paste content with invisible Unicode; normalize NFC at every input boundary}}.
- LRN-11: {{e.g., Vendoring {{lib}} was cheaper than tracking its breaking releases; keep it vendored}}.
- LRN-12: {{e.g., The batch importer must commit in chunks of ≤500 or the DB lock queue starves the API}}.
- A learning graduates to an invariant (§4) when breaking it would be catastrophic rather than expensive.
- When you discover a new learning, add it here in the same PR that exploited it.
- Never delete a learning because the code that taught it is gone; the trap usually returns wearing new code.

## 8. CAVEATS AND FOOTGUNS — HANDLE WITH CARE

- These are places where the obvious action is the wrong action.
- CAV-1: {{e.g., `scripts/reset.sh` drops the production schema if $ENV is unset. Check $ENV. Twice}}.
- CAV-2: {{e.g., The test suite passes with stale snapshots; regenerate snapshots before trusting green}}.
- CAV-3: {{e.g., Module {{X}} looks dead but is loaded reflectively by {{Y}}; grep will not find the caller}}.
- CAV-4: {{e.g., The config in `defaults.yml` is overridden by env vars in prod; the file lies to you}}.
- CAV-5: {{e.g., Two functions named `serialize` exist; the one in `legacy/` is the one prod actually uses}}.
- CAV-6: {{e.g., The staging DB is a manual copy from March; absence of data there proves nothing}}.
- CAV-7: {{e.g., `id` in the events table is not unique; uniqueness is (id, region). Yes, really}}.
- CAV-8: {{e.g., The build caches aggressively; a clean build is required after touching codegen}}.
- CAV-9: {{e.g., The rate limiter counts per-IP, and our biggest customer is behind one NAT}}.
- CAV-10: {{e.g., Deleting a user must go through `retire_user()`; raw deletes orphan billing records}}.
- Anything marked `# LOAD-BEARING` in code is here for a reason documented in this section or §7.
- If you find a footgun not listed here, list it before you do anything else.

## 9. DEPENDENCY POLICY

- Every dependency is a marriage: you inherit its bugs, its politics, and its release schedule.
- Default answer to a new dependency is no; the burden of proof is on inclusion.
- A dependency earns its place by replacing ≥{{N}} lines of code we'd otherwise own, or by covering a domain we must not hand-roll (crypto, timezones, parsing untrusted input).
- Never hand-roll cryptography, no matter how simple it looks.
- All versions are pinned exactly; upgrades are deliberate commits with changelogs read, not lockfile churn.
- Read the changelog before any upgrade; "patch" versions have shipped breaking changes to us before ({{ref}}).
- Dependencies with a single maintainer and no releases in 18 months are flight risks; list them in §21.
- Transitive dependency count matters as much as direct count; check the full tree.
- License check is mandatory: nothing incompatible with {{project license}} enters the tree.
- Dev-only dependencies get more slack than runtime dependencies, but not infinite slack.
- Removing a dependency is always worth a small amount of extra code.

## 10. DATA AND STATE RULES

- Data outlives code; every schema decision is a decade-long commitment.
- Store facts, not interpretations: {{e.g., store the event, derive the status}}.
- Never destroy information in a migration; transform forward, keep the original until verified.
- Every migration must be reversible or explicitly marked as a point of no return with sign-off.
- Nullability is a design decision, not a default; every nullable column needs a reason.
- Soft-delete is the default for user-visible entities; hard-delete requires the retention policy in §14.
- All external input is hostile until validated at the boundary; internal code trusts validated types only.
- State machines are explicit: valid transitions live in {{location}}, and nothing bypasses them.
- Derived data must be rebuildable from source-of-truth data with one documented command.
- Backfills run through the same code path as live writes, or they will drift.
- Clock skew is real; never compare timestamps from two machines for ordering — use {{ordering mechanism}}.

## 11. ERROR HANDLING DOCTRINE

- Errors are part of the interface, not an afterthought.
- Fail loudly and early in development; fail safely and legibly in production.
- Never swallow an exception without a comment stating why silence is correct.
- Every error message must contain enough context to act on it without a debugger: what, where, which entity.
- Distinguish the three families everywhere: user error, environment error, and our bug. Handle them differently.
- Our bugs crash noisily (with telemetry); user errors return guidance; environment errors retry with backoff.
- Retry only idempotent operations; see LRN-9.
- Partial failure is the hard case; every batch operation must define what half-done means and how to resume.
- Timeouts on every external call, no exceptions; the default timeout is {{value}}.
- Error paths get tests too; the sad path is where users actually meet us.

## 12. TESTING DOCTRINE

- Tests encode intent; a test that only mirrors implementation is weight, not safety.
- Test behavior at boundaries, not private internals; internals must stay refactorable.
- Every bug fix ships with the test that would have caught it. No exceptions.
- Every invariant in §4 has a named test; find them via {{convention, e.g., tests tagged @invariant}}.
- Flaky tests are treated as outages: quarantined same day, fixed or deleted within a week.
- Deleting a test requires the same scrutiny as deleting the feature it guards.
- Coverage percent is a smell detector, not a goal; 100% coverage of the wrong assertions is zero safety.
- The fast suite must stay under {{N}} seconds or people stop running it; guard that budget.
- Golden/snapshot tests need human-readable diffs, or updates become rubber stamps (see CAV-2).
- Property-based tests guard the parsers and serializers; example-based tests guard the business rules.
- Do not mock what you own; refactor for testability instead. Mock only true externals.

### 12.1 Commit and change doctrine

- One commit = one intent; mixing a fix with a refactor destroys bisectability.
- The commit message states why; the diff already states what.
- Reference the learning, caveat, or decision line (LRN-n, CAV-n) that a change relates to.
- Refactors ship as separate commits with zero behavior change, provable by the untouched test suite.
- Never commit commented-out code, debug prints, or TODOs without an owner and a §21 entry.
- A PR too large to review carefully is too large to merge; split it.
- Generated files are either committed with their generator pinned, or never committed — no half measures ({{which policy applies here}}).
- Force-pushing shared branches is forbidden; history on {{main branch}} is append-only.
- Revert first, diagnose second; a broken {{main branch}} outranks any in-flight investigation.

## 13. PERFORMANCE BOUNDARIES

- Performance work without a measurement is fiction; profile first, always, in a realistic environment.
- The budgets that matter: {{e.g., p95 request < 200 ms, cold start < 2 s, memory < 512 MB per worker}}.
- Inside budget, optimize for legibility; outside budget, optimize with a profile in hand.
- The known hot paths are: {{list}}. Changes there require a before/after benchmark in the PR.
- The known cold paths are everything else; do not micro-optimize them, ever.
- Big-O matters at our scale for {{specific operations}}; constant factors matter for {{other operations}}.
- The pathological input that hurts us is {{e.g., one user with 100k items}}, not average load — design for the skew.
- Caching is a last resort after algorithmic fixes; every cache added must state its invalidation story here.
- Premature optimization and premature pessimization are both trendslop; measured is the only mode.

## 14. SECURITY AND PRIVACY

- We hold user data in trust; convenience never outranks that trust.
- Threat model in one line: {{e.g., malicious input from any user; curious insiders; compromised dependencies}}.
- Secrets never enter the repo, logs, error messages, or test fixtures. Never means never.
- All secrets come from {{secret mechanism}}; a secret found anywhere else is an incident.
- Authentication and authorization are separate questions; check both, in that order, at {{boundary}}.
- Authorization is default-deny; every endpoint states who may call it or it does not ship.
- Log events, not payloads; PII in logs is a breach, not a debugging aid.
- The PII inventory lives in {{location}}; touching any field on that list triggers the review in §19.
- Data retention: {{policy, e.g., raw events 90 days, aggregates indefinitely, deletion requests honored in 30 days}}.
- Injection is not a solved problem; parameterize queries, escape output, validate uploads — everywhere, every time.
- Dependency CVEs are triaged within {{N}} days; the scanner config lives at {{location}}.
- Security shortcuts taken under deadline pressure are recorded in §21 with a payoff date.

## 15. NAMING AND VOCABULARY — THE UBIQUITOUS LANGUAGE

- Words are architecture; a wrong name propagates a wrong model.
- The domain glossary is authoritative; code, docs, and UI use these words and no synonyms.
- {{Term A}}: {{precise meaning; what it is NOT}}.
- {{Term B}}: {{precise meaning; what it is NOT}}.
- {{Term C}}: {{precise meaning; what it is NOT}}.
- "{{Ambiguous word}}" is banned in identifiers; it has meant three different things historically (see LRN-{{n}}).
- Renaming a domain term is a migration, not a refactor; it touches docs, UI, and this file.
- New concepts get named here before they get coded; unnamed concepts breed synonyms.
- Prefer domain words over technical words at boundaries users see: {{e.g., "workspace" not "tenant_ctx"}}.

## 16. INTERFACES AND COMPATIBILITY

- Every public interface is a promise; we keep promises even when they are inconvenient.
- Public surface is exactly {{definition, e.g., what is exported from /api}}; everything else may change without notice.
- Breaking changes require: deprecation notice, migration path, {{N}} releases of overlap, then removal.
- Deprecated code is marked with the removal version at deprecation time, not "later."
- Wire formats and file formats are versioned explicitly; readers accept N and N-1, writers emit N only.
- Unknown fields in input are preserved, not dropped; forward compatibility is a courtesy we extend.
- Error responses are part of the API contract; changing an error code is a breaking change.
- Internal convenience never justifies widening the public surface; see §3.

## 17. OPERATIONAL KNOWLEDGE — HOW IT LIVES IN THE WORLD

- The system runs at {{environment summary}}; local dev diverges from prod in these known ways: {{list}}.
- The one command to build and test everything from a clean checkout: {{command}}. If it stops working, that is a P1.
- Deploys are {{mechanism}}; rollback is {{mechanism}} and must complete within {{N}} minutes.
- Every feature must be deployable dark and rollback-safe; schema and code deploy separately.
- The dashboards that tell the truth are {{location}}; the metric that pages a human is {{metric}}.
- The first three things to check during an incident: {{one}}, {{two}}, {{three}}.
- Known operational quirks: {{e.g., the Tuesday batch job doubles DB load 02:00–03:00 UTC}}.
- Runbooks live at {{location}}; an alert without a runbook is noise and gets deleted or documented.
- Backups are only real if restores are tested; last verified restore: {{date}} — keep this line current.

## 18. AGENT PROTOCOL — WHEN YOU ARE UNSURE

- Uncertainty is information; act on it, do not paper over it.
- Before any non-trivial change, state your understanding of intent in one sentence and check it against §2 and §4.
- If the task conflicts with a non-goal or invariant, stop and surface the conflict; do not creatively reinterpret.
- Prefer reading three more files over guessing once.
- Never fabricate: not APIs, not config keys, not historical reasons. "I don't know" is a valid, useful output.
- If you catch yourself justifying a change with "best practice" and nothing else, stop — that is trendslop knocking.
- Confidence must be proportional to evidence from this repo, not from training-set averages.
- Ambiguity in requirements is resolved by asking, or by choosing the interpretation with the smallest blast radius.
- Every session that changes code ends with: what was learned, what surprised you, what belongs in this file.
- You may propose changes to this file; you may not silently violate it.

### 18.1 First-session orientation order

- Read in this order before your first edit: this file → §4 invariants twice → {{entrypoint file}} → {{core module}} → the tests for whatever you will touch.
- Then run the one command from §17 and confirm green before changing anything; a red baseline poisons every conclusion after it.
- Locate the source of truth for the state you will touch (§5) before reading any code that consumes it.
- Grep for the domain terms (§15) involved in your task; the hits map the real blast radius.
- Check §21 for open debts overlapping your task; touching a debt's territory means addressing or explicitly deferring it.
- Only after all of the above are you calibrated enough to estimate the task.

## 19. CHANGE PROTOCOL FOR THIS FILE

- This file changes slowly and deliberately; it is the constitution, not the newspaper.
- Additions require: the concrete event that motivated the line, in the commit message.
- Removals require: proof the reason no longer holds, plus a tombstone entry in §20.
- Amendments to §4 (invariants) require maintainer sign-off; agents propose, humans ratify.
- Reversing a decision requires engaging the original reason recorded here — Chesterton's fence is the house rule.
- Keep lines atomic; when a line grows compound, split it.
- Keep the file under ~400 lines; when it grows past that, distill — merge duplicates, promote patterns, cut the stale.
- Distillation must never silently drop a learning or caveat; compress the wording, keep the trap.
- Review this file quarterly against reality; a normative file that drifts from reality trains readers to ignore it.
- Date of last full review: {{date}} by {{who}}.

## 20. DECISION LOG — TOMBSTONES AND TURNING POINTS

- Format: DATE — decision — reason — what we gave up.
- {{2024-06}} — {{Adopted X over Y}} — {{reason}} — {{trade-off accepted}}.
- {{2024-11}} — {{Reverted async pipeline}} — {{complexity exceeded benefit at our scale}} — {{throughput headroom}}.
- {{2025-03}} — {{Froze public API v1}} — {{three integrators in production}} — {{freedom to rename endpoints}}.
- {{2025-09}} — {{Removed plugin system}} — {{zero external plugins in 18 months}} — {{hypothetical extensibility}}.
- Dead ends already explored (do not re-explore without new facts): {{list, e.g., GraphQL gateway, CRDT sync, mono-to-poly repo split}}.
- Each dead end above has its post-mortem at {{location}}; read it before proposing the idea again.

## 21. THE DO-NOT-FORGET LIST — LIVE DEBTS AND OPEN LOOPS

- This section is the working memory between sessions; keep it honest and keep it short.
- DEBT-1: {{e.g., auth middleware has a temporary bypass for the demo tenant — remove by {{date}}}}.
- DEBT-2: {{e.g., the export path is O(n²) past 10k rows; fine today, not fine at 3× growth}}.
- DEBT-3: {{e.g., dependency {{lib}} is unmaintained; exit plan sketched at {{location}}}}.
- RISK-1: {{e.g., single point of failure: the token-signing key rotation has never been exercised}}.
- OPEN-1: {{e.g., undecided: retention policy for audit logs — blocked on legal}}.
- Every entry has an owner or a date; entries with neither are wishes, and wishes get deleted.
- Paying off a debt means deleting its line here and adding the learning to §7 if it taught one.

## 22. THE COVENANT

- The code will be rewritten many times; the intent recorded here must survive every rewrite.
- Fashion is rented; understanding is owned. This file is what we own.
- When this file and your instincts disagree, distrust your instincts first — they were trained on the average repo, and this is not the average repo.
- Protect the core promise (§2), honor the invariants (§4), and leave a trail (§18–21).
- Everything else is negotiable.