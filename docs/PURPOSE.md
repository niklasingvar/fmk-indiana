---
status: draft
purpose: Explain why Indiana exists and what loop it improves.
approval: approved
approved_file_length: 50 rows
---

# PURPOSE

## Why this exists
- Improve Human–AI–Computer interaction at the point of review.
- Raise productivity, review speed, and review precision.
- Coding agents emit a lot; humans review slowly. Close that gap.

## Core idea
- The review loop:
  1. Agent emits code, markdown, slides etc.
  2. Human tags lines with `::` markers ([COMMANDS.md](indiana/IN_COMMANDS.md)) — fast reactions, no essays.
  3. Indiana monitors the repo, compiles every marker
  4. Agent reads the compiled payload through Indiana's MCP surface, or user copies it as fallback.


- Brilliance: Indiana exposes the same compiled payload to agents and humans — markers expand to prompts, context travels with them.


## Unfair advantage
- Leverage existing coding agents.



## The products
- [Indiana](indiana/IN_PRD.md) — the server. Monitors repos, compiles `::` markers, exposes the payload through MCP, copies the bundle as fallback. Owns the markers end to end. CLI + menulet.
- [Menulet](menulet/MENULET_PRD.md) — a UI view onto Indiana: monitored folders, one-click copy. Shows, never computes.
- [Casablanca](casablanca/CASABLANCA_OVERVIEW.md) — the editor: rich inline markdown editing, visual/presentation support as features. Emits `::` markers from rendered views; Indiana still owns them.

## Direction
- This file is the wedge; the destination is [VISION.md](../VISION.md). Concern map: [MENTAL_MODEL.md](../MENTAL_MODEL.md).
- Built in steps — see [ACTION_PLAN.md](../ACTION_PLAN.md). Rome isn't built in a day.
- See [GOAL.md](GOAL.md) for what success looks like.
