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
- Auto-run flag: `-a` / `--auto` immediately after the token (before the message) asks the daemon to run this marker at once over ACP — `::fix -a banana`. Honored only on agent directives that act directly (`::fix`, `::elaborate`, `::prompt`); on any other kind a leading `-a` is ordinary message text. The daemon claims the marker (`::fix[id:working] -a …` — the flag stays in source; the bracket status gates dispatch), dispatches, and the agent resolves it by deleting the line. Full lifecycle: [IN_AUTORUN.md](IN_AUTORUN.md).
- Numeric group flag: a positive `-<number>` before the message assigns the marker to that repo-wide batch — `::fix -1 banana`. All marker kinds may be grouped; the flag is metadata and does not enter the compiled message. A menulet Run dispatches every member as one ACP turn; Copy renders only that group. `-a` and a numeric group may coexist, in either order, but `-a` still dispatches that marker individually.
- Agent persona flag: `-<agent-name>` before the message tags the marker for a named agent defined in `.indiana/agents/<name>/SYSTEM_PROMPT.md` — `::fix -mike create this task`. A single letter works while exactly one agent starts with it (`-m` → mike); on a collision the letter stops resolving and stays message text, full names still work. Copying or dispatching an agent batch prepends that agent's system prompt instead of the default. A numeric group and an agent flag are mutually exclusive — the second flag of the other dimension stops the flag scan and stays message text. `-a` may coexist with either.
- Inside a code fence → ignored.
- Frontmatter property comments use `# frontmatter.<key> ::<cmd> [message]`; their inline scope is the named property.
- File opt-out: `::ignore` in the frontmatter (or as a first-line comment in files without one) removes the whole file from scanning. It is a directive, not a marker kind — details in [IN_SCAN.md](IN_SCAN.md).
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
- User task: `::action` (Human queue), `::todo` / `::task` (Agent queue) — tracked into the Chief of Staff tracker ([COS_PRD.md](../chief-of-staff/COS_PRD.md)).

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
| `::a` | `::action` | message | user task | Human-queue item; tracked, never executed by agents. |
| `::td`, `::task` | `::todo` | message | user task | Agent-queue task; tracked with an origin backlink. |
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
- `::note <msg>` → passed through as user context.
- `::todo <msg>` → msg + do it, mark the tracker line done, delete the marker line ([COS_PRD.md](../chief-of-staff/COS_PRD.md)).
- `::action <msg>` → msg + human-queue item, do not execute.
- `::delete [msg]` → "Take action on this and delete the targeted content. Confirm with the user before deleting." + msg.
- `::prompt [msg]` → "Run the code agent directly on this." + msg. (Auto-calling the code agent ships later.)

## Decided
- Pure reactions (`::hate`, `::love`, `::keep`) take no message — keeps tagging fast (PURPOSE: no essays).
- Numeric labels are scoped to one monitored root. The same `-1` in another root is a different group.
- Agent flags resolve only against the owning root's `.indiana/agents/` roster; in a root with no agents, `-m` is ordinary message text. Mike (chief of staff) and Lisa (CTO / architecture) are scaffolded by default.
- Prompt wording is tunable content, not frozen here ([IN_PRINCIPLES.md](IN_PRINCIPLES.md): content is data). The intent above is the contract; phrasing can change.
