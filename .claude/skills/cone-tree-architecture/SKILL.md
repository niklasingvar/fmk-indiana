---
name: cone-tree-architecture
description: Uphold the cone-shaped tree structure when doing domain modelling or structuring knowledge — creating, moving, splitting, or restructuring files in docs/, .indiana/context-model/, .indiana/chief-of-staff/, or any hierarchical doc set. Use when the user asks "where does this file/fact go", "restructure the docs", "model this domain", "is this the right place", or when a change adds a new markdown file, a new doc folder, or a new level to an information tree.
---

# Cone-shaped tree architecture

The principle named in [FUNDAMENTALS.md](../../../FUNDAMENTALS.md) (tier: Principles — how
knowledge is structured). The full ruleset for the context-model instance lives in
`.indiana/context-model/CONTEXT-MODEL.md`; the concern map for this repo's docs is
[MENTAL_MODEL.md](../../../MENTAL_MODEL.md). This skill is the portable core: apply it to any
information tree, link to those files instead of restating them when working inside their scope.

## The shape

A knowledge tree is a cone standing on its tip:

- One apex. A single entry point — a schema, index, or purpose file. Depth 1 is the domain
  model: the few names that carve the space. Every reader starts there, every file is reachable
  from there.
- Widens downward. Each level has more, smaller, more specific nodes than the level above.
  Depth correlates with specificity and inversely with stability: the higher a file, the rarer
  it changes.
- One parent per fact (SSOT). Every fact has exactly one home file; everywhere else it appears
  as a link, never a copy. Promote, never fork.
- Conflicts resolve upward. When two files disagree, the more stable (higher) level wins — fix
  the lower file toward it or explicitly promote the contradiction.
- Knowledge flows up only by compression. Raw detail enters at the bottom (inbox/learnings
  level) and earns its way up by being confirmed and compressed. Never author directly into the
  apex what a leaf has not proven.

## Shape violations (name them when you see them)

- Bulge: a level narrower than the one below it hanging off one oversized file — split the fat
  file along its domain lines.
- Chimney: deep single-child chains (`a/b/c/one-file.md`) — collapse the empty levels.
- Pancake: a flat root with dozens of siblings and no domain level — introduce the depth-1
  names and route files under them.
- Orphan: a file no index or parent links to — it does not exist; index it or delete it.
- Mirror: a file restating what one grep or one file-read reveals — negative value; delete and
  link to the source instead.

## Pre-flight checklist — before adding or moving a file

1. Which level? State the file's specificity and expected change rate; place it at the depth
   that matches (stable+general = high, volatile+specific = low).
2. Which parent? Exactly one home; the path alone should reveal the file's concern
   ([MENTAL_MODEL.md](../../../MENTAL_MODEL.md): the path decides the concern).
3. Does the apex know? Add the one-line pointer to the index/entry file in the same change.
4. Does anything it says already have a home? If yes: link, do not restate. If it must be
   restated for readability, one clause plus the link, non-normative.
5. Does the tree still widen? If the new file makes a level wider than its children can
   justify, or deepens a chimney, restructure first.

## Per-file mechanics

- Frontmatter on every file (status, purpose, approval — plus whatever the local schema
  demands). A file without frontmatter is invisible.
- One line = one thing; a line needing two sentences is two lines.
- Reads are routed, never scanned: a reader descends apex → domain → leaf; if the index line
  is not enough to decide the descent, fix the index line.
- Budgets are law: prefer editing an existing file over creating one; when a size ceiling is
  hit, compress — never raise the ceiling.
