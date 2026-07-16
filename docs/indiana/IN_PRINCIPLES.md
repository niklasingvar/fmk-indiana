---
purpose: The architectural invariants that keep Indiana cheap to change. Every other spec assumes these.
max_lines: 55
status: draft
approval: pending
---

# IN_PRINCIPLES — invariants

> What must stay true so Indiana never rots. Engine: [IN_SCAN.md](IN_SCAN.md). Markers: [IN_COMMANDS.md](IN_COMMANDS.md). Identity: [IN_IDENTITY.md](IN_IDENTITY.md). Faces: [IN_MCP.md](IN_MCP.md).

## Source is the only truth
- Markdown is the record. Everything else — index, counts, payload, bundle — is a cache derived from it.
- Test: delete `.indiana/`, rescan, state is byte-identical. If it isn't, something holds state it shouldn't.
- This is why `block_history` is out: text-over-time is not derivable from source, so it cannot exist.
- Carve-out: user config (the monitored-folders list, [IN_DAEMON.md](IN_DAEMON.md)) is input, not a cache. It legitimately persists; it is not derived state and is not what this rule governs.
- Carve-out: the copy cursor (`~/.indiana/copied.json`) is interaction history — a record of "which markers were already copied." It is optional convenience state, not a cache of source. Deleting it degrades safely: `--latest` falls back to copy-all. Source truth is untouched. Append-only by design: a copy may scan one subfolder, so garbage-collecting the cursor against a single scan would silently drop identities for every file outside it. Growth is bounded by distinct markers ever copied; delete the file to reset.
- Carve-out: repo-local `.indiana/` command templates ([IN_FOLDER.md](IN_FOLDER.md)) are user-authored input. Deleting them changes prompt wording, not marker state.
- Carve-out: `.indiana/chief-of-staff/tasks.md` + `log.md` ([COS_PRD.md](../chief-of-staff/COS_PRD.md)) are Chief of Staff state. tasks.md is hybrid: marker-captured rows re-derive from source on rescan; hand-added rows and state edits are input and are lost with the file. log.md is interaction history like the copy cursor — deleting it loses history only. The `indiana task`/`indiana log` CLI is the read/write face.

## One marker table drives everything
- The grammar — short/long form, kind, arg, compiled prompt, identity, default scope — is declared once.
- Parser, compiler, identity, and the menulet read that one table. None re-encode the marker set.
- Adding a marker is one row, not an archaeology dig across files. The set keeps growing; make growth cheap.

## The write path is a single chokepoint
- All mutation of user files goes through one function with one contract: byte-preserving, atomic, mtime-guarded, idempotent.
- Nothing else writes. The one dangerous thing Indiana does has the smallest possible blast radius.
- ID injection and the `done`/`failed` mark are the only writes — both flow through this one door.

## Core computes, faces render
- The core owns all domain logic: parse, resolve scope, count, compile. MCP, CLI, and menulet only expose or display.
- A face never counts, parses, or assembles a prompt. "The menulet never counts" ([IN_PRD.md](IN_PRD.md)) generalizes to every face.
- MCP ([IN_MCP.md](IN_MCP.md)) is an agent-readable face over the same compiled payload as `indiana copy`.
- Why: duplicated logic is two things to fix for every change. Keep faces dumb.

## Stateless per line
- Parsing a line is a pure function of the line plus fence state — the one declared cross-line bit.
- No other line coupling. Keeps the parser trivially testable and the scan parallelizable.

## Content is data, not code
- Compiled-prompt wording is product content, tuned often. It lives as templates/data, not in engine code.
- Changing how `::hate` reads must not mean recompiling the scanner.
- Marker grammar is global; folder-local templates tune prompt wording per monitored root.
- The system prompt is the same kind of content: authored in `crates/core/templates/system_prompt.md`, versioned in frontmatter, overridable per root as `.indiana/SYSTEM_PROMPT.md`. Changing what every agent reads first must not mean editing Rust.

## Templates have one home
- `crates/core/templates/` is the single authoring source for everything a monitored root starts with: full `indianas/<command>/prompt.md` files, the versioned `system_prompt.md`, and meta folder seeds. Embedded at compile time, written verbatim.
- A unit test (`test_embedded_templates_match_marker_table`) fails if a template's frontmatter drifts from its marker TABLE row. Edit the template and the marker row together.
- A unit test (`test_system_prompt_names_fundamentals`) pins the embedded system prompt to the fundamental names every agent must see.
- This repository's own `.indiana/` is a dogfood instance, not a source. It may diverge from the templates freely — like every other monitored repo, it is user input that tunes wording for existing kinds ([IN_FOLDER.md](IN_FOLDER.md)). To change what users receive, edit `crates/core/templates/`.

## Love becomes direction
- A `::love` marker means more than preserve this instance.
- Agent abstracts the liked pattern into [IN_PRINCIPLES.md](IN_PRINCIPLES.md) or the nearest relevant spec.
- The result is directional intent, not a copied example.

## Spec is the contract; code conforms
- Direction lives in these specs; code proves it. When they disagree, the spec wins or the spec changes — never silent drift.
- Each requirement maps to a test (the E-criteria). Drift is a missing test, not bad luck.

## Decided
- The marker table lives embedded in the core — single static binary, no external config to drift.
- Faces query the core for it. One copy, never two to sync. Markers are product grammar; prompt wording is user-tunable content.
