---
purpose: Define the `::` marker grammar Indiana parses and compiles.
max_lines: 60
status: draft
approval: pending
---

# COMMANDS — marker grammar

> Single source of truth for the `::` markers. One indiana is one `::` command in the source. They belong to [Indiana](IN_PRD.md): it monitors repos, parses these markers, and compiles them on `indiana copy`. Agents read them over [IN_MCP.md](IN_MCP.md); the [Menulet](../menulet/MENULET_PRD.md) only displays Indiana's output; [Casablanca](../casablanca/CASABLANCA_OVERVIEW.md) emits markers from rendered views but never parses or compiles them.

## Syntax
- `::<cmd>` at column 0, or inline at end of a content line.
- Short or long form, e.g. `::h` == `::hate`.
- A message follows the token only when the kind takes one (see the table; pure reactions take none).
- Auto-run flag: `-a` / `--auto` immediately after the token (before the message) asks the daemon to run this marker at once over ACP — `::fix -a banana`. Honored only on agent directives that act directly (`::fix`, `::elaborate`, `::prompt`); on any other kind a leading `-a` is ordinary message text. The daemon claims the marker (`::fix[id:working]`, flag consumed), dispatches, and the agent resolves it. Full lifecycle: [IN_AUTORUN.md](IN_AUTORUN.md).
- Inside a code fence → ignored.
- Frontmatter property comments use `# frontmatter.<key> ::<cmd> [message]`; their inline scope is the named property.
- Why `::` (not `[]`): survives every markdown parser; `rg '^::'` has zero false positives.

## Identity and scope
- Identity: [IN_IDENTITY.md](IN_IDENTITY.md). Scope: [IN_SCOPE.md](IN_SCOPE.md).
- `::action` / `::todo` carry a completion state (`done` / `failed`) and an ID: [IN_IDENTITY.md](IN_IDENTITY.md). On-disk syntax: [IN_LINE.md](IN_LINE.md).

## Types
- Agent directive: `::fix`, `::elaborate`.
- Agent explains: `::question`.
- Agent gated directive: `::delete`.
- Agent runs directly: `::prompt`.
- Reaction: `::hate`, `::love`, `::keep`.
- User context: `::note`.
- User task: `::action`, `::todo`.

## The set
| Short | Long | Arg | Kind | Meaning |
|-------|------|-----|------|---------|
| `::q`, `::?` | `::question` | optional message | question | User asks about this; no message means user does not understand. |
| `::h` | `::hate` | — | reaction | User dislikes this. User does not explain. |
| `::l` | `::love` | — | reaction | User likes this; preserve the pattern. |
| `::k` | `::keep` | — | reaction | Freeze; do not change. |
| `::f` | `::fix` | optional message | agent directive | Agent fixes this; message refines how. |
| `::e` | `::elaborate` | optional message | agent directive | Agent takes action and elaborates the change. |
| `::n` | `::note` | message | user context | Note for the user. |
| `::a` | `::action` | message | user task | Task for the user to log. |
| `::td` | `::todo` | message | user task | Alias for action. |
| `::d` | `::delete` | optional message | agent gated directive | Agent deletes targeted content; checks in with the user before acting. |
| `::p` | `::prompt` | optional message | agent runs directly | Auto-calls the code agent to act on this prompt (behavior ships later). |

## Compiled prompts on copy
- Prompt wording can come from repo-local templates ([IN_FOLDER.md](IN_FOLDER.md)). Defaults:
- `::hate` → "The user tagged this as hate. Explain to the user in a numbered list why he hates it." — the why is generated, not user-written.
- `::love` → "The user loves this. Keep it and apply the same pattern. Abstract the direction and update [IN_PRINCIPLES.md](IN_PRINCIPLES.md) or the nearest relevant spec so it stays directional."
- `::keep` → "Do not modify this."
- `::fix [msg]` → "Fix this." + msg.
- `::elaborate [msg]` → "Take action on this and elaborate the change." + msg.
- `::question [msg]` → "The user asks: <msg>. Answer it." If no message: "The user does not understand this. Explain it."
- `::note <msg>` / `::action <msg>` / `::todo <msg>` → passed through as user context / user task.
- `::delete [msg]` → "Take action on this and delete the targeted content. Confirm with the user before deleting." + msg.
- `::prompt [msg]` → "Run the code agent directly on this." + msg. (Auto-calling the code agent ships later.)

## Decided
- Pure reactions (`::hate`, `::love`, `::keep`) take no message — keeps tagging fast (PURPOSE: no essays).
- Prompt wording is tunable content, not frozen here ([IN_PRINCIPLES.md](IN_PRINCIPLES.md): content is data). The intent above is the contract; phrasing can change.
