---
status: draft
purpose: Map each place in the repo to its single concern — build docs, product specs, templates, instances — and fix one name per concept.
approval: pending
---

# MENTAL_MODEL — four concerns, four places

> The path decides the concern. If a file's job is not decidable from its path, the file is misplaced. The same word never names two concerns.

## The four concerns

fmk-indiana is simultaneously a thing being built, a thing being specified, a thing that ships content, and a tool installed on this same laptop to build other apps. One place per concern:

| Concern | Question it answers | Place |
|---|---|---|
| build docs | how do we build this? | `ALLCAPS.md` at root and `docs/` root, `rules/` |
| product specs | what is this? | `docs/<product>/`, code in `crates/` conforms |
| templates | what does every user receive? | `crates/core/templates/` |
| instances | the tool in use | `.indiana/` in monitored repos |

### Build docs
- Guide Niklas + agents toward building the right thing.
- `VISION.md` (destination), `docs/PURPOSE.md` (wedge), `docs/GOAL.md`, `ACTION_PLAN.md`, `MENTAL_MODEL.md`, `CLAUDE.md`, `docs/AGENT_*.md`, `TODO.md`, `rules/`.
- Never shipped, never read by Indiana at runtime.

### Product specs
- Define each product: scope, invariants, behavior. Spec wins or spec changes ([IN_PRINCIPLES.md](docs/indiana/IN_PRINCIPLES.md)).
- `docs/ARCHITECTURE.md` (system), `docs/indiana/IN_*.md`, `docs/menulet/MENULET_*.md`, `docs/casablanca/CASABLANCA_*.md`, `docs/context-model/CM_*.md`, `docs/chief-of-staff/COS_*.md`.

### Templates
- The single authoring source for everything a monitored root starts with: `crates/core/templates/` — full `indianas/<command>/prompt.md` files (written verbatim) plus meta seeds for `context-model/` and `chief-of-staff/`.
- Embedded into the binary at compile time; lives inside `crates/core/` because a packaged crate build carries only the crate dir.
- Not documentation. Specs describe the fix command; the template file *is* the fix command. Changing a word here changes what users receive.
- Guard: `test_embedded_templates_match_marker_table` pins each template's frontmatter to its marker TABLE row.

### Instances
- A monitored repo's `.indiana/` after `indiana add`: templates materialized, then diverging — tuned prompts, growing context-model, live todos.
- User data. Upgrades never overwrite it ([IN_FOLDER.md](docs/indiana/IN_FOLDER.md): refresh adds missing, replace is explicit and destructive).
- The one unrecoverable concern: deleting an instance loses accumulated knowledge.

## Dogfood
- This repo's own `.indiana/` is an instance like any other: fmk-indiana installed on fmk-indiana.
- It may diverge from the templates freely — that is dogfooding, not drift. No test pins it.
- To change what users receive, edit `crates/core/templates/`, never `.indiana/`.

## Naming convention
- One canonical name per concept — paths, specs, code, chat. Codenames are flavor and live only in [VISION.md](VISION.md).
- Nimbalyst is not a product of ours: external open source (nimbalyst.com), vendored at `crates/casablanca/nimbalyst/` as reference only. Never name our things after it.
- Visual/presentation support is a feature set inside casablanca, not a named product.

| Concept | Canonical | Doc prefix | Retired aliases |
|---|---|---|---|
| the whole system | fmk-indiana | — | — |
| marker engine + daemon + CLI | indiana | `IN_` | — |
| menu-bar face | menulet | `MENULET_` | Bangalore |
| the editor | casablanca | `CASABLANCA_` | nimbus, visualviewer |
| per-repo memory | context-model | `CM_` | Boxydoc, meta model |
| human/agent focus layer | chief-of-staff | `COS_` | montmartre |

- Build doc: `ALLCAPS.md` at repo root or `docs/` root. No prefix.
- Spec: `docs/<canonical>/<PREFIX>_TOPIC.md`. Folder name = canonical name, nothing appended (`docs/indiana/`, not `docs/indiana-engine/`).
- Template: lowercase, under `crates/core/templates/` only.
- Instance: `.indiana/` — dogfood in this repo, user data everywhere else.

## The test
- Given any path, the concern is decidable without opening the file.
- Given any concept, one `rg` for the canonical name finds every mention; a `rg` for any retired alias finds nothing.
- Deleting build docs loses direction. Deleting specs loses the contract. Deleting templates breaks what ships. Deleting an instance loses a user's accumulated knowledge — the one unrecoverable place.
