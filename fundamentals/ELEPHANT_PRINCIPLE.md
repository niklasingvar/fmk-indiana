---
status: draft
purpose: Eat it one bite at a time — small files, small problems, small loops, hard ceilings.
approval: pending
---

# ELEPHANT PRINCIPLE

## Definition
- Anything too big to hold is cut into pieces small enough to finish and review in one pass.

## Rules
- Small files, small problems, small loops.
- Every file carries a max-rows ceiling.
- When a ceiling is hit, the answer is compression or a split — never a ceiling raise.

## Test
- A file or task that cannot be reviewed in one sitting is too big.

## Incorporation
- This repo: `max_lines` frontmatter on the `docs/indiana/IN_*.md` specs; the budgets section of [docs/context-model/CONTEXT-MODEL.md](../docs/context-model/CONTEXT-MODEL.md) §10.
- System prompt: the context-model seed carries the budgets into every monitored repo's loops.
- settings.json: `maxRowsPerFile` in `.indiana/casablanca/settings.json` — the fundamental names the parameter, settings tune the value per repo.
