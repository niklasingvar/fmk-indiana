---
purpose: Specify what an indiana targets — the span of content a marker carries into the bundle.
max_lines: 65
status: draft
---

# IN_SCOPE — what an indiana targets

> Scope is the content an indiana points at. Markers: [IN_COMMANDS.md](IN_COMMANDS.md). Engine: [IN_SCAN.md](IN_SCAN.md).

## Why scope
- A marker is useless without the content it refers to.
- The resolved span travels into the copy bundle, so the agent sees exactly what the human tagged.
- Humans tag fast; scope must be inferable from where the marker sits, not spelled out.

## The spans
- Inline: marker at end of a content line targets that line.
- Next-row: marker alone on a line targets the row after it (the next block until a blank line).
- Section: marker on a heading targets that section, until an equal or higher heading.
- Range: marker opens a span that reads until a terminator — for larger, multi-block selections.

## Resolution
- Position decides span: end-of-line vs. own-line vs. heading.
- One indiana resolves to one span. No nesting.
- The span is captured at scan time and frozen into the bundle.

## Edge cases (decided)
- Inline on a heading: marker at end of a `#` line stays inline — it targets the heading text, not the section. Position decides; to scope the section, put the marker alone above it (next-row) or use section rules below.
- Next-row block: the block is contiguous non-blank lines until the next blank line. A whole list, or a whole blockquote, is one block — not per-item.
- Section headings: ATX (`#`) only. Setext (`===` / `---` underline) is not a section anchor — a marker on the text line is inline; the underline carries nothing. A `>` quoted heading is content, not a section.
- Nested sections: `###` under `##` is a sub-span. A marker on `### Bar` scopes to `### Bar` until the next equal-or-higher heading — its own narrow span, not the parent.

## Decided
- Range deferred to a later phase. When built, an `::end` line closes the most recent open range — line-oriented, survives parsers.
- Spans never cross file boundaries. A span is bounded by its file (folder-as-architecture; one indiana, one file).
- Most-specific wins. An inline indiana inside a section keeps its own narrow span; the section does not swallow it.
- Build order: inline + next-row first (clearest), section second (needs heading-level tracking), range last (needs `::end`). Mirrors [../PHASES.md](../PHASES.md) Phase 3.
