---
id: purpose.glossary
layer: purpose
status: active
owner: human
purpose: The ubiquitous language — one term, one meaning, used identically in code, docs, and UI.
upstream: [purpose]
review_by: 2027-01-05
updated: 2026-07-05
---

# GLOSSARY

- **Indiana** — the headless command engine and CLI; the logic layer. Never runs its own agent.
- **indiana (lowercase)** — one command definition: a folder `indianas/<command>/` with its prompt template.
- **Casablanca** — the editor; rich markdown editing with visual/presentation support as features. NOT a separate viewer product. (retired alias: nimbus)
- **Context-model** — this tree; per-repo memory. (retired aliases: Boxydoc, meta model)
- **Chief of Staff** — human/agent focus management; two queues (Human TODOs, Agent TODOs). NOT status-tracking. (retired alias: montmartre)
- **Menulet** — the macOS menu-bar surface. Visualizes and triggers; never computes. (retired alias: Bangalore)
- **Loop** — one full pass: annotate → collect → agent executes → artifact and tree both updated.
- **Command / marker** — a `::` annotation in a file (`::fix`, `::hate`, ...); maps to an instruction set, not just an edit request.
- **Operator** — the human working the loop; used instead of "user" in all normative writing.
- **Wedge / Destination** — the shipped `::` review loop vs. the full workspace vision; one system, two zoom levels.
- Codenames live in vision docs only; paths and specs use canonical names — see [MENTAL_MODEL.md](../../../MENTAL_MODEL.md).
- Retired aliases (nimbus, COS, Boxydoc, Bangalore) are banned in new writing; finding one is a lint-worthy learning.
