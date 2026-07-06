---
id: rules.global
layer: rules
status: active
owner: shared
purpose: Cross-cutting rules for how every artifact in this repo must be created, regardless of type.
upstream: [architecture, purpose]
review_by: 2026-10-05
updated: 2026-07-05
---

# GLOBAL RULES

- Every file created in this repo carries frontmatter; a file without frontmatter has no lifecycle and no trust.
- Canonical component names only (see [GLOSSARY.md](../purpose/GLOSSARY.md)); retired aliases are defects.
- Write for the operator, not for a generic audience; the operator's taste lives in [OPERATOR.md](../preferences/OPERATOR.md) and is read before any diagnostic loop.
- Link, never copy: any fact needed from another file appears as a link; a copied paragraph is an SSOT violation to fix on sight.
- Agent edits stay minimal-diff: touch what the marker targets, resist adjacent "improvements" — record the urge in learnings/INBOX.md instead.
- Every artifact change must be explicable in one sentence of intent; that sentence goes in the log entry.
- Template-based artifacts respect the content/design split (INV-7): pick the file class before editing, never both in one loop.
