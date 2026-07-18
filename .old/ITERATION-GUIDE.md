---
status: draft
purpose: How the operator iterates — running the indiana-loop, updating the context-model schema, and updating command templates.
approval: pending
---

# ITERATION-GUIDE — how we loop

> Concern map: [MENTAL_MODEL.md](MENTAL_MODEL.md). Loop wiring: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Current focus: [FOCUS.md](FOCUS.md).

## The loop, one pass

1. Open the repo in Casablanca.
2. Tag commands: inline `::` markers in markdown, or click elements in an HTML preview — annotations land as marker lines in the `page.html.md` sidecar.
3. Press `Copy all`.
4. Paste into any coding agent.
5. The payload's preamble drives three outcomes:
   - the artifact is edited per each marker's prompt;
   - `.indiana/context-model/` learns — `log.md` gets one entry per command, `::hate`/`::love`/`::note` add an atomic entry to `learnings/INBOX.md`;
   - `.indiana/chief-of-staff/focus.md` captures todos (`::todo`) and status updates.
6. The editor refreshes from the watcher; verify, tag again, repeat. Each loop should be shorter.

## Update the context model here

- Authoring source: [files/CONTEXT-MODEL.md](files/CONTEXT-MODEL.md) — the full schema. Edit it first.
- Shipped seed: `crates/core/templates/context-model/CONTEXT-MODEL.md` — a compressed version of the authoring source; recompress after schema edits ([CM_PRD.md](docs/context-model/CM_PRD.md) owns this contract).
- Live instance: `.indiana/context-model/` — dogfood, adopted from the prepared tree 2026-07-06; may diverge freely, upgrades never overwrite it.
- Tree maintenance (consolidation, promotion, lint) follows the schema's own sections 7–9; `::lint` is schema-promised but not yet a marker — see the gap entry in `.indiana/context-model/architecture/DECISIONS.md`.

## Update command templates here

- Single authoring source: `crates/core/templates/indianas/<command>/prompt.md` — embedded into the binary; changing a word changes what users receive.
- `test_embedded_templates_match_marker_table` pins each template's frontmatter to its marker TABLE row; run `cargo test -p indiana-core` after edits.
- The loop preamble every payload carries lives in `crates/core/templates/preamble.md`.
- This repo's `.indiana/indianas/` is a dogfood instance: refresh is explicit (`indiana templates refresh`), never automatic.
- New commands (e.g. the missing `::lint`) go through the create-indiana-command flow: template → parser row → compiler → counts → docs → tests.

## Keep commands and context model in sync

- The schema's write-back protocol (§7) and the template bodies must agree on targets: INBOX for diagnostics, `focus.md` for tasks, `log.md` for every command.
- After editing either side, re-check the three targets above; drift gets a DECISIONS.md entry, not a silent fix.

## Demo script (show the loop to anyone)

1. `npm run dev` in `crates/casablanca`; choose this repo as vault.
2. Open any `.html` file; click an element; add `::fix` with a message, `::hate` bare, `::todo` with a task.
3. Open a markdown doc; type an inline `::note why this matters`.
4. `Copy all`, paste into Claude Code.
5. Show the receipts: the artifact diff, `grep "^## \[" .indiana/context-model/log.md | tail -3`, the new INBOX entry, the unchecked item in `.indiana/chief-of-staff/focus.md`.
