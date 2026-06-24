---
name: Copy Latest — cursor design
overview: "Design a scalable cursor for `indiana copy --latest` (copy only markers not yet copied). Neither TODO option (timestamp / last-seen ID) works alone: markers carry no timestamp, and only `::action`/`::todo` have stable IDs. Chosen: a persisted seen-set of position-independent marker identities, diffed against the live scan in core. Plan only — no implementation."
todos:
  - id: core-identity
    content: Add a position-independent marker identity fn in core (tracked → injected id; ephemeral → content fingerprint, computed, never written).
    status: pending
  - id: core-latest-filter
    content: Extend CompileOptions with a copied-identity set; exclude those markers in compile, alongside the kind filter.
    status: pending
  - id: face-cursor-store
    content: Load/save the copied-set in ~/.indiana/ (CLI + daemon), pass to core, persist after a successful copy.
    status: pending
  - id: cli-copy-latest
    content: Wire `indiana copy --latest`; compose with `--kind`.
    status: pending
  - id: docs
    content: Document the cursor carve-out (IN_PRINCIPLES), ephemeral-fingerprint distinction (IN_IDENTITY), command + acceptance (TODO).
    status: pending
  - id: verify
    content: Core + CLI tests, including the twice-run acceptance and --kind composition.
    status: pending
isProject: false
---

# Copy Latest — cursor design

## Depends on
- The kind-filter plan ([`plans/cli_copy_filters_805e7251.plan.md`](cli_copy_filters_805e7251.plan.md)) introduces `compile_with_options` + `CompileOptions { kind }`. This plan extends that struct. Land kind-filter first, or land both together.

## The design problem
- `--latest` = "markers since the last copy". A cursor must answer "is this marker new since I last copied?" per marker.
- TODO offers two cursor shapes; both fail against the data model:
  - Last-copied timestamp — markers carry no time. Source has only file mtime: whole-file granularity, recopies untouched markers in any edited file, clock/FS dependent. Not per-marker, not precise.
  - Last-seen marker ID — only `::action`/`::todo` are tracked with IDs ([IN_IDENTITY.md](../docs/indiana/IN_IDENTITY.md)). The other 7 kinds are ephemeral, addressed by position. A single "last ID" also has no ordering (IDs are random tokens, not monotonic) — "since" is undefined.
- So a single scalar cursor cannot cover all kinds. The generalization that does: a set of stable per-marker identities.

## Chosen approach — seen-set of marker identities
- Persist the set of marker identities already copied. `--latest` renders current markers whose identity is not in the set. After a successful copy, add the copied markers' identities to the set.
- This is "last-seen marker ID" extended to every kind: tracked markers use their real ID; ephemeral markers use a computed fingerprint.

## Marker identity (core)
- One core fn, identity per `Located`:
  - Tracked (`::action`/`::todo`): the injected ID. Already stable across edits and moves ([IN_IDENTITY.md](../docs/indiana/IN_IDENTITY.md)).
  - Ephemeral (the other 7): content fingerprint = hash of `path` + `kind` + `raw_token` + `message` + `scope_content`. Deliberately excludes line number.
- Consequences, all acceptable and documented:
  - Move a marker within a file (line shifts) → same identity → not re-copied.
  - Edit a marker's text/scope → new identity → re-appears in `--latest` (changed directive = new). Correct.
  - Move to another file (path changes) → new identity. Acceptable.
- Critical distinction: the ephemeral fingerprint is computed in memory for the cursor only. It is never written to source. Preserves IN_IDENTITY ("IDs everywhere would be noise") and the single write chokepoint.
- Placement: a new `crates/core/src/cursor.rs` (folder-as-architecture), or extend `id.rs`. Decide at implementation.

## Filter placement (core computes, faces render)
- Add `copied: Option<HashSet<String>>` to `CompileOptions`. `compile_with_options` drops markers whose identity is in `copied`, same pass as the kind filter.
- The diff lives in core; the face only supplies the set and persists it. Keeps CLI/MCP/menulet dumb (IN_PRINCIPLES "core computes, faces render").

## Persistence (face)
- `~/.indiana/copied.json` — JSON array of identity strings. Mirrors `config.json` (serde_json pretty, `INDIANA_HOME` override for tests — already in [`paths.rs`](../crates/indiana/src/paths.rs)).
- One global set. Identities are path-qualified, so multiple monitored folders share one file without collision.
- Read before compile, write after a successful clipboard set.

## Semantics
- What advances the cursor: every successful copy adds the identities of the markers it actually delivered (the post-filter set), not the whole scan. So `copy --kind action` records only the actions it copied; a later `copy --latest` still surfaces never-copied notes/hates. "Since last copy" = "not yet copied". This is what makes `--kind action --latest` compose correctly (TODO line 20).
- First run / missing cursor: empty set → `--latest` copies everything, then writes the set. (Acceptance run-twice still holds: second run sees a full set, copies nothing.)
- GC: on write, intersect the stored set with current scan identities before adding the new ones. Drops fingerprints of deleted/edited markers so the file stays bounded. A marker that returns byte-identical after deletion counts as new — acceptable.
- Plain `indiana copy` (no `--latest`): still advances the cursor (records what it copied). Only `--latest` changes what is rendered.

## Principle reconciliation — needs a decision
- IN_PRINCIPLES "source is the only truth": delete `.indiana/`, rescan, state byte-identical. A copy cursor is not derivable from source — it records an external event, the same smell that rules out `block_history`.
- The existing carve-out covers user config (input, not a cache). The cursor is a third category: interaction history.
- Recommend: extend the carve-out — the cursor is optional convenience state, not a cache of source. Deleting it degrades safely (`--latest` falls back to copy-all); source truth is untouched. Document it explicitly in IN_PRINCIPLES so this is not silent drift.
- Rejected alternative: write a `copied` mark into source like `done`/`failed`. Only works for the 2 tracked kinds, pollutes source for the other 7, and multiplies writes. No.

## Implementation steps (where, not how)
- [`crates/core/src/cursor.rs`](../crates/core/src/cursor.rs) (new): `identity(&Located) -> String`; tracked vs ephemeral fingerprint.
- [`crates/core/src/compile.rs`](../crates/core/src/compile.rs): add `copied` to `CompileOptions`; exclude in `compile_with_options` before `compile_marker`.
- [`crates/indiana/src/main.rs`](../crates/indiana/src/main.rs): add `copy --latest`; load set, pass options, persist post-copy set.
- New face store beside [`config.rs`](../crates/indiana/src/config.rs) (e.g. `copied.rs`) for load/save of `copied.json`.
- [`crates/indiana/src/mcp.rs`](../crates/indiana/src/mcp.rs): optional — same `copied` option for parity (TODO defers MCP latest; note, don't force).

## Verification
- Core:
  - Identity stable when a marker's line shifts; differs when message/scope edited; tracked uses injected id.
  - `compile_with_options` with a copied-set excludes exactly those markers.
  - GC: identities absent from the current scan are dropped.
- CLI (with `INDIANA_HOME` tmp):
  - Acceptance: `indiana copy --latest` twice — first copies N, second copies 0.
  - Composition: fixture with `::hate`, `::action`, `::note`; `copy --kind action --latest` copies the action only and does not consume the hate/note for a later `copy --latest`.
  - Add a marker between two `--latest` runs → second copies exactly the new one.
  - Delete `copied.json` → `--latest` falls back to copy-all.

## Docs to update (end-of-task)
- [IN_PRINCIPLES.md](../docs/indiana/IN_PRINCIPLES.md): cursor carve-out (interaction state, safe to lose).
- [IN_IDENTITY.md](../docs/indiana/IN_IDENTITY.md): ephemeral fingerprint is computed for the cursor only, never written.
- [TODO.md](../TODO.md): record the chosen approach; resolve the "timestamp or last-ID" question.

## Open questions
- Cursor scope: one global `copied.json`, or per-monitored-root? Recommend global (path-qualified identities already disambiguate).
- Should plain `indiana copy` advance the cursor, or only `--latest`? Recommend every copy advances (matches "since the last copy").
