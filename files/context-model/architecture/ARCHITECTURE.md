---
id: architecture
layer: architecture
status: active
owner: shared
purpose: The system's shape, boundaries, and the invariants no loop may break.
upstream: [purpose]
review_by: 2027-01-05
updated: 2026-07-05
---

# ARCHITECTURE

- The shape in one line: five separable components — Indiana (logic), Context-model (memory), Casablanca (visual), Chief of Staff (focus), Menulet (surface) — each valuable alone, compounding together.
- Each component can be built, tested, and killed independently; a change that entangles two of them is fighting the shape.
- Everything is files in the folder: `.indiana/` holds commands, this tree, and chief-of-staff; Casablanca keeps `settings.json` per project. No hidden state elsewhere.
- Indiana is harness-agnostic: it collects and compiles `::` payloads; execution belongs to the operator's agent (Claude Code, Codex, Cursor).
- The handoff evolves in three phases — copy-paste → MCP pull → auto-run — each removing one manual step, never skipping ahead.
- Casablanca is self-built (Electron + Lexical, `crates/casablanca`); Nimbalyst is vendored at `crates/casablanca/nimbalyst/` as reference patterns only — see [DECISIONS.md](DECISIONS.md).
- Presentations separate content from design: content files iterate cheaply, design files stay untouched by content loops, templates seed rough drafts.

## Invariants (violating one is a critical bug even if everything still runs)

- INV-1: Indiana never executes an agent itself; the harness is always external.
- INV-2: All configuration and memory is plain files inside the repo folder — readable by any agent, versionable by git, editable by a human.
- INV-3: `::` markers are inert text until collected; a file with markers must remain valid as its own format.
- INV-4: A loop never destroys operator content; agent edits are always reviewable via git diff.
- INV-5: Every diagnostic command (`::hate`, `::love`, `::note`) writes back to this tree; a diagnostic loop that leaves the tree untouched is incomplete.
- INV-6: The Menulet visualizes and triggers; it never computes.
- INV-7: Content edits never rewrite design files, and design edits never rewrite content files, in template-based artifacts.
