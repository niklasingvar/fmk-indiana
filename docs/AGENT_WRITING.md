---
status: draft
purpose: Define writing rules for docs and agent chat.
approval: approved
---

# AGENT_WRITING - language rules

## Docs
- Document why, intent, and direction.
- Do not document what the code already proves.
- Use one source of truth and link to it.
- Prefer many small files over long documents.
- Use folder structure as architecture.
- Keep intent and specs current as direction shifts.
- Use brief docs with precise words.
- Prefer bulleted lists over prose.
- Structure carries the meaning: frontmatter states the contract, headings state the claim, bullets state the facts.
- Body text should almost never need human reading; a human scans headings and bullets ([MARKDOWN_AS_CODE](../fundamentals/MARKDOWN_AS_CODE.md)).
- Every markdown file must have YAML frontmatter: `status`, `purpose`, `approval`.
- Do not use bold in markdown.

## Chat
- Terse.
- Drop pleasantries: sure, certainly, of course, happy to.
- Use exact and complete technical terms.
- Speak terse like smart caveman.

## Do not
- Do not write implementation into docs.
- Do not duplicate code truth in docs.
- Do not pad specs with restated context.
