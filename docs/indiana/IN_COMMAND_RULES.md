---
status: draft
purpose: Rules for indiana commands.
approval: pending
---

# Indiana command rules

## Command types

Only these `command_type` values are valid:

- `operator_directive`: caller owns intent, scope, and target; agent executes the declared task.
- `agent_directive`: agent acts on content directly.
- `agent_gated_directive`: agent acts on content through a gate, used for destructive operations.

## Command boundaries

- Command frontmatter defines a contract between caller and agent.
- Commands describe the action shape, not the agent's private reasoning.
- The caller owns intent, scope, and target.
- The agent owns faithful execution of the declared command.
- Gating belongs at command boundaries, not inside recursive agent doubt.
- Destructive behavior is a command category, not a reason to invent extra responsibility.
- File content targets use editor language: `line`, `section`, `file`.
- Structural behavior belongs in fields; interpretation belongs in prompts only when necessary.

## System update

- Every command carries `system-update: right`.
- The agent must update the correct, relevant part of the system: the nearest spec, doc, or file that owns the truth for the change.
- Do not scatter updates across unrelated parts of the system.
