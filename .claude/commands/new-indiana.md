---
name: new-indiana
description: Guide adding a new Indiana `::` marker command end-to-end, from the `.indiana` template through parser, compiler, counts, docs, and tests.
---

Use the `create-indiana-command` skill to add a new Indiana command.

Read `.indiana/indianas/` first — this repo's templates are the authoring source for default command wording. If the user names a command that already has a draft under `.indiana/indianas/<command>/prompt.md`, normalize that draft to the standard frontmatter contract (`status`, `purpose`, `approval`, `command`, `command_type`, `message`, then a `#` heading, then the prompt body) instead of starting from scratch.

Treat any text after this command as the command name or intent (for example `/new-indiana delete`). If the long token, short token, message contract, tracked flag, `command_type`, or prompt wording is missing, ask one focused question before editing. Do not guess the prompt wording silently — confirm it with the user.

Follow the skill's Apply and Verify steps exactly. Keep `.indiana/indianas/<command>/prompt.md` and `crates/core/prompts.toml` in sync — the `test_repo_indianas_match_embedded_defaults` test enforces it.
