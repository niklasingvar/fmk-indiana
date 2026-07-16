---
status: draft
purpose: Markdown is source code — parseable, lintable, diffable; structure carries the meaning.
approval: pending
---

# MARKDOWN AS CODE

## Definition
- Markdown is treated as source code: parseable, lintable, diffable.

## Rules
- One line = one thing; a line that needs two sentences is two lines.
- Structure carries the meaning: frontmatter states the contract, headings state the claim, bullets state the facts.
- Body text should almost never need human reading — a human scans headings and bullets; agents parse the rest.
- No bold in docs; prefer bulleted lists over prose ([docs/AGENT_WRITING.md](../docs/AGENT_WRITING.md)).

## Test
- If lint cannot check it, restructure it until lint can.
- If a human must read body prose to get the point, the structure failed.

## Incorporation
- This repo: [docs/AGENT_WRITING.md](../docs/AGENT_WRITING.md) enforces it on every doc an agent writes.
- System prompt: `::` markers rely on it — one marker per line is what makes feedback parseable at all.
- settings.json: `maxRowsPerFile` bounds file size so the scan-don't-read promise holds.
