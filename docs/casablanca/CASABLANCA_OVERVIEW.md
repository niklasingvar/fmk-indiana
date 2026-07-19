---
status: draft
purpose: Casablanca product overview — what the editor is and where it sits in the system.
approval: pending
---

# CASABLANCA — Overview

> The implemented feature inventory lives in [CASABLANCA_PRD.md](CASABLANCA_PRD.md). Decisions and constraints: [CASABLANCA_ARCHITECTURE.md](CASABLANCA_ARCHITECTURE.md). Forward-looking roadmap: [CASABLANCA_ROADMAP.md](CASABLANCA_ROADMAP.md). Open questions: [CASABLANCA_QUESTIONS.md](CASABLANCA_QUESTIONS.md).

> The visual layer of [VISION.md](../../VISION.md). Why the system exists: [PURPOSE.md](../PURPOSE.md). System shape: [ARCHITECTURE.md](../ARCHITECTURE.md). Roadmap: [ACTION_PLAN.md](../../ACTION_PLAN.md) Phase 1.

## What it is
- The editor: open a repo, edit markdown inline as rich text (WYSIWYG, no edit/preview split), see artifacts as what they are — documents as documents, slides as slides.
- Annotating emits ordinary `::` markers into the source file. Casablanca is a face; [Indiana](../indiana/IN_PRD.md) owns the markers.
- A top-bar `Copy all` icon hands the compiled Indiana payload to the clipboard — the iterate loop without a terminal.

## Per-repo settings
- Repo-local settings live in `.indiana/casablanca/settings.json` ([IN_FOLDER.md](../indiana/IN_FOLDER.md)) — a committable JSON bag, so a repo carries its own editor preferences and the CLI can read/edit them: `indiana casablanca get|set|settings|path`.
- The editor reads the keys it knows and ignores the rest. Known editor keys: `color` (project identity color, overrides the global registry) and `theme` (`light` | `dark` — no in-app toggle; edit the file). The Indiana daemon reads `autoRun` and optional `model` for repo-local agent dispatch ([IN_AUTORUN.md](../indiana/IN_AUTORUN.md)).
- App-global state (the project list, which project is active) stays in the editor's own `userData` config, not per-repo.
