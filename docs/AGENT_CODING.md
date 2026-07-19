---
status: draft
purpose: Define coding constraints for agents changing this repo.
approval: approved
---

# AGENT_CODING - code rules

## Simplicity first
- Minimum code that solves the problem.
- Nothing speculative.
- No features beyond what was asked.
- No abstractions for single-use code.
- No flexibility or configurability that was not requested.
- No error handling for impossible scenarios.
- If 200 lines could be 50, rewrite it.

## Surgical changes
- Touch only what is required.
- Clean up only your own mess.
- Do not improve adjacent code, comments, or formatting.
- Do not refactor things that are not broken.
- Match existing style, even if you would do it differently.
- If unrelated dead code appears, mention it instead of deleting it.
- Remove imports, variables, or functions that your change made unused.
- Do not remove pre-existing dead code unless asked.
- Every changed line should trace to the user request.

## Goal-driven execution
- Convert the task into verifiable goals.
- Add validation: write tests for invalid inputs, then make them pass.
- Fix a bug: write a test that reproduces it, then make it pass.
- Refactor: ensure tests pass before and after.
- Upgrade a dependency across a major version: read its breaking changes and cover the behavior we rely on with a test. Silent API narrowing is real: chokidar v4 dropped glob support and the watcher watched nothing, without any error.
- For multi-step tasks, state a brief plan with verification for each step.
- Strong success criteria let the agent loop independently.
- Weak success criteria require clarification.

## CLI first
- CLI is the primary interface.
- Every feature ships as a CLI command first.
- Menulet replicates CLI features 1:1: shows, never computes.
- Every subcommand maps 1:1 to a core function.
- CLI never re-parses, re-counts, or re-compiles.
- Adding a marker kind in `markers.rs` must not require a CLI change unless a new flag is explicitly requested.
- CLI argument names match core enum or struct field names.
- No magic string translations.
- A new core entry point must be wired into the CLI in the same change.
- `indiana help` must stay accurate after every change.
