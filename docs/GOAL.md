---
status: draft
purpose: Define what success looks like for Indiana.
approval: pending
---

# GOAL

> See [PURPOSE.md](PURPOSE.md) for *why*. This is *what success looks like*.


For a user; check target audinece
- [Indiana](indiana/IN_PRD.md) scans the folder and exposes an agent-readable payload: generated prompts + tagged context.
- Human tags lines with `::` markers ([COMMANDS.md](indiana/IN_COMMANDS.md)) while reviewing.
- Agent reads the payload through [Indiana MCP](indiana/IN_MCP.md); human copy remains fallback.
- The folder tree alone explains the system (folder as architecture).

## Measured by
- Review speed ↑ (time from agent output → human decision).
- Review precision ↑ (fewer missed / wrong calls).
- Tokens per agent message ↓.

## Achieved in steps
- We don't ship the end state at once.
- First win: the server up, scanning all markdown and listing `::` markers from the CLI.
- Full sequence in [PHASES.md](PHASES.md).
