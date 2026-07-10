---
status: draft
purpose: Floor plan for agents working in this repo.
approval: approved
---

# CLAUDE.md - floor plan

## Start here
- [MENTAL_MODEL.md](MENTAL_MODEL.md) - the four concerns and naming convention
- [docs/PURPOSE.md](docs/PURPOSE.md) - why this exists
- [docs/GOAL.md](docs/GOAL.md) - what success looks like
- [ACTION_PLAN.md](ACTION_PLAN.md) - sequencing and roadmap (supersedes docs/PHASES.md)

## Product maps
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - system-level components, loops, state stores
- [docs/indiana/IN_ARCHITECTURE.md](docs/indiana/IN_ARCHITECTURE.md) - engine shape and ownership boundaries
- [docs/indiana/IN_PRINCIPLES.md](docs/indiana/IN_PRINCIPLES.md) - Indiana invariants
- [docs/indiana/IN_TEST.md](docs/indiana/IN_TEST.md) - requirement-to-test map
- [docs/menulet/MENULET_PRD.md](docs/menulet/MENULET_PRD.md) - menulet product contract
- [docs/casablanca/CASABLANCA_OVERVIEW.md](docs/casablanca/CASABLANCA_OVERVIEW.md) - Casablanca product contract
- [docs/casablanca/CASABLANCA_PRD.md](docs/casablanca/CASABLANCA_PRD.md) - Casablanca implemented-feature inventory

## Agent rules
- [docs/AGENT_OPERATING.md](docs/AGENT_OPERATING.md) - read-first loop, assumptions, end-of-work
- [docs/AGENT_CODING.md](docs/AGENT_CODING.md) - simplicity, CLI-first, surgical edits, verification
- [docs/AGENT_WRITING.md](docs/AGENT_WRITING.md) - docs and chat language rules
- [docs/AGENT_COMMIT.md](docs/AGENT_COMMIT.md) - small focused commits

## Always
- Read the files above that match the task before editing.
- State assumptions when uncertain.
- If multiple interpretations exist, present them.
- If a simpler approach exists, say so.
- Casablanca is Electron-only. Run `cd crates/casablanca && npm run dev` and verify the separate Electron window; `http://localhost:5173` is renderer-only and cannot load `window.api`. If the Electron window itself reports a missing preload bridge, inspect its DevTools and preload path. Do not add browser mocks unless browser support is explicitly requested.
- End by documenting learnings, caveats, or new principles in the relevant markdown file.
- Point out consistency issues, violations, caveats, or simpler paths.