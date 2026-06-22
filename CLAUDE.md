# CLAUDE.md

## Always 
### Read first
- [PURPOSE.md](PURPOSE.md) — why this exists
- [GOAL.md](GOAL.md) — what success looks like
- [PHASES.md](PHASES.md) — sequencing / roadmap
### End with
- Documenting the learnings, caveats, new principles in the relevant markdown file
- Point out consistency issues, violations, caveats or simpler way of doing things
### Commit
- Commit often: small, focused commits, one logical change each.
- Commit after every passing step/milestone — don't batch unrelated work.
- Keep docs and code in separate commits.

## Principles
- Document WHY, INTENT and DIRECTION, never what the code already proves.
- Single source of truth; link to other docs.
- Many small files over long documents.
- Folder as architecture (group files into folders)
- Keep intent and specs current as direction shifts.
- Terse in chat. Brief in docs; precise words.
- Bulleted lists over prose.
- All markdown files must have YAML frontmatter (status, purpose, approval)
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.

## Writing rules
- Never use bold in markdown
- Drop pleasantries: sure, certainly, of course, happy to
- Technical terms: always exact and complete
- Speak terse like smart caveman

## Don't
- Don't write implementation.
- Don't duplicate code's truth in docs.
- Don't pad specs with restated context.

# Wrting code;

## Simplicity First
Minimum code that solves the problem. Nothing speculative.

No features beyond what was asked.
No abstractions for single-use code.
No "flexibility" or "configurability" that wasn't requested.
No error handling for impossible scenarios.
If you write 200 lines and it could be 50, rewrite it.
Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## Surgical Changes
Touch only what you must. Clean up only your own mess.

When editing existing code:

Don't "improve" adjacent code, comments, or formatting.
Don't refactor things that aren't broken.
Match existing style, even if you'd do it differently.
If you notice unrelated dead code, mention it - don't delete it.
When your changes create orphans:

Remove imports/variables/functions that YOUR changes made unused.
Don't remove pre-existing dead code unless asked.
The test: Every changed line should trace directly to the user's request.

## Goal-Driven Execution
Define success criteria. Loop until verified.

Transform tasks into verifiable goals:

"Add validation" → "Write tests for invalid inputs, then make them pass"
"Fix the bug" → "Write a test that reproduces it, then make it pass"
"Refactor X" → "Ensure tests pass before and after"
For multi-step tasks, state a brief plan:

1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.