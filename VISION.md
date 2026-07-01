# AI-Native Workspace with Learning Loops and an Opinionated, LLM-Friendly Architecture

*Working codenames:*
- **Indiana** · headless command engine and CLI
- **Casablanca** · editor and visual layer
- **Montmartre** · project management — ADHD style
- **Boxydoc** · context model and memory
- **Bangalore** · app menulet

---

## The Problem

The current way of working with AI on real artifacts — presentations, documents, code, apps — is broken in two directions.

The chat direction is exhausting: you describe what you want, read a wall of markdown, copy-paste it somewhere else, then come back to the chat and try to explain *where* in the artifact your feedback applies ("the third paragraph on the second slide, no not that one..."). Pointing at things through prose is cumbersome, and every round trip through chat is an expensive, slow feedback loop.

The agent direction isn't better: plan, run the agent, read a review dump, give feedback, repeat. The review artifacts are painful to read and the loop is just as long.

Meanwhile, the tooling itself has terrible UX. Power users end up writing prose documents inside code editors because that's where their tools live. Non-developers get chat interfaces with no ability to see or touch the actual artifact. Nobody gets to stay close to the thing they're actually making.

**The core insight: the content is the only thing that matters.** Everything else — the prompting, the loops, the reviewing — is overhead. The mission is to collapse that overhead so the human can stay focused on the artifact itself.

## The Thesis

Stay closer to the artifact. Ideally, remove chat from the system entirely.

Instead of describing feedback in a chat window, you annotate the artifact directly — in the file, or on a rendered view of it — using a lightweight, fully configurable inline command language. An agent picks up those annotations, acts on them in the background while you keep working, and the artifact updates in place.

The unit of work is a **folder**. A folder is a mission: it holds the artifact, the context, and the configuration for how AI should behave in that project. This is deliberately aligned with where the world is going — every serious coding agent already operates against a folder. We don't fight that; we build on it.

## The Dream: Compound Knowledge Through Loops

The dream is that I, as an operator, iterate on my artifacts without ever entering a chat.

Every loop runs through the same machinery: the prompt templates in `.indiana/` are kept aligned with Boxydoc — the rule set for *how things should be created* in this repository. Those templates do two jobs at once. They direct the coding agent to the right material — what to read, what rules apply, what the artifact is for. And they instruct the agent to write back what it learned: updated metadata, updated context, updated rules.

The result is that `.indiana/` becomes a strict, hierarchical information tree — a domain tree — that the agents continuously maintain and refine as a side effect of doing the work. Every iteration doesn't just improve the artifact; it improves the *system's understanding of the project*. Feedback given once is never given twice.

This is the unique selling point of the whole workflow: **knowledge compounds through the loops**. Chat-based workflows are stateless — every session starts from zero and every preference must be restated. Here, the loops are the memory. Together with the domain-specific language and the no-chat workspace, this is what makes the system more than the sum of its tools.

## The Components

The system is deliberately modular — five separable pieces that are valuable alone and compound together. Separating them means each can be built, tested, and killed independently.

### Indiana — the command engine (the logic layer)

A domain-specific language for inline feedback, plus the server that acts on it.

You write commands directly in your files: `::fix`, `::hate`, `::love`, `::note`, or anything you define yourself. Each command maps to an instruction set that tells the LLM what mode to enter. `::hate` doesn't just mean "change this" — it means "diagnose *why* the user hates this, explain it back, and update the project's context model so it never happens again."

Everything lives in a `.indiana/` folder inside the repository:

- `commands/` — one file per command (`fix.md`, `design.md`, `amend.md`...). Fully configurable per project, so `::design` means one thing in a dev repo and another in a presentation project.
- `context/` — Boxydoc's home (see below): the rules, purpose, and accumulated learnings for this repo. Every command execution reads it; commands like `::hate` write back to it.

**Crucially, Indiana is harness-agnostic.** It never runs its own agent — the collected commands are handed to Claude Code, Codex, Cursor, whatever you already use. Their tokens, their quota, their harness. Indiana operates against the folder; so do they. It complements them rather than competing with them, which sidesteps the fatal problem of asking users to buy tokens in a new system.

How the handoff happens evolves in three phases, each one removing a manual step:

- **Phase 1 — copy-paste.** `indiana copy-all` collects every `::` payload across the repo into one prompt. You paste it into your agent and hit enter. Degraded mode by design — and honestly a great workflow already.
- **Phase 2 — MCP.** Claude Code connects to the Indiana server via MCP: the agent pulls the payload itself, no clipboard involved. Click run in Casablanca or Bangalore, and the loop fires.
- **Phase 3 — auto-run.** The Indiana server monitors the folder and dispatches commands as you write them. You mark `::fix` on a line, keep reading, and it's fixed by the time you scroll back. Pausable, of course — sometimes you want to stack up a bulk of annotations and fire them in one go.

### Boxydoc — the context model (the memory layer)

Boxydoc is the project's brain: the rule set for how things should be created in this repository, and the accumulated knowledge from every loop that has run.

It holds the purpose of the project, the operator's preferences, the structural rules for artifacts, and the learnings extracted from feedback commands ("the user hates business jargon", "titles should be questions, not statements"). It is organized as a strict hierarchical information tree, and it lives as plain files inside `.indiana/` for now — readable by any agent, versionable by git, editable by a human. It's a separate component conceptually (so it can grow, or be killed, on its own), but not a separate folder yet.

The contract is bidirectional: Indiana's prompt templates read from Boxydoc before every execution, and write back to it after. Boxydoc is what turns isolated command executions into compound knowledge — it is the component that makes the Dream possible.

### Casablanca — the editor and viewer (the visual layer)

Cursor is a terrible place to look at a presentation. Chat is a terrible place to point at slide three. Casablanca renders artifacts as what they actually are — slides as slides, documents as documents — with annotation directly on the rendered view. Mark a section as hate, click run, and the feedback flows into Indiana.

Artifact types, roughly in order of attack:

1. **Presentations** — rendered slide decks with annotation boxes.
2. **Documents / text** — a proper reading, writing, and annotating view for markdown. Writing happens *in* the rendered view — no split between edit mode and preview mode.
3. **Code** — shown raw; devs want raw. Annotation and inline commands, not prettification.
4. **Excalidraw** — a visual canvas as a first-class artifact type; sometimes the fastest way to think is to draw.
5. **Apps** — web only, always. The DOM gives the system a way to know what happened to the UI; native doesn't.

The spectrum Casablanca navigates: **raw/native** mode (maximum speed and flexibility, crap UX) versus **rendered** mode (great UX, but human edits require version handling — a real technical problem, parked for now).

*Notes: inspired by Nimbalyst (open source). A local `settings.json` per project keeps viewer configuration alongside the repo — same philosophy as `.indiana/`: configuration is files in the folder.*

### Montmartre — project management, ADHD style

Not status-tracking; attention-tracking. Montmartre stores two queues in a local SQLite database: **Human TODOs** (what the operator needs to decide, review, or provide) and **Agent TODOs** (work items the agents can pick up autonomously).

Indiana reads the Agent TODO queue and executes against it; completed loops and open questions flow back as Human TODOs. The point is a clean division of labor: the human queue stays short and decision-shaped, the agent queue drains in the background, and at any moment the answer to "what should I be doing right now?" is one glance away.

### Bangalore — the menulet

A lightweight, always-visible macOS menu bar app: the smallest possible surface for the whole system. It shows the current focus, per-project status (what's running, what's waiting on you), and offers one-click copy of the collected Indiana payload — so the copy-paste-into-your-agent loop never requires opening a terminal.

Bangalore is deliberately dumb: it visualizes Montmartre and triggers Indiana, nothing more. Its simplicity is the feature.

---

## How It Works: The Presentation Flow

1. You, the operator, select a repository (folder).
2. You create a `brief.md` in Casablanca and type what you want: "five slides pitching X."
3. Laziness kicks in — you mark it `::elaborate`.
4. You hit run — in Phase 1 that means `indiana copy-all` and a paste into Claude Code; from Phase 2 the agent pulls the payload itself over MCP. Either way, the prompt template turns your one-liner into an elaborated `brief.md`.
5. You hit `::prompt` to render the slides. The agent scaffolds from a **template**: pre-built, pretty-enough slide components, with **content separated from design**.
6. Casablanca renders the deck. You annotate directly on it: `::hate` on a couple of things, `::fix` on the title, a free-text note on slide four.
7. You hit run. Indiana diagnoses the feedback, updates content files (cheap, fast) without touching design files, logs the change to the relevant context files, and updates Boxydoc so the next iteration already knows your preferences.
8. Repeat — but each loop is shorter, and there are fewer of them.

The template-first, content/design-separated architecture is the key economic move: rough drafts are token-cheap (copy a template, fill it in), content iterations don't rewrite design, and design iterations don't rewrite content. Per-change cost may be higher than raw markdown chat, but the promise is **fewer total loops** from nothing to good enough — and thanks to Boxydoc, each loop is smarter than the last.

## What This Becomes

Individually, these are tools. Together — a folder as mission, context that accumulates, inline commands, background execution, a viewer, a focus layer — this is a **work management system** for the AI-native era. Not project management as status-tracking, but the actual environment where work happens: you pick a folder, the mission and context live in it, any AI can connect to it, and the feedback loop between you and the artifact is as short as writing two colons.

## Who It's For

**Now: me.** One user, power-user mode, dogfooded on real projects (starting with the card game). Developers are the natural first audience — the annotate-specific-lines problem is already solved for them and they'll feel the speed immediately.

**Later: the non-dev who's ready for agentic workflows** — the manager writing a pitch deck, the person who wants to see and touch their document. Bangalore and buttons are the bridge. Same system underneath; the config files are just invisible to them.

## Deliberate Non-Goals (For Now)

- **No own harness.** Building one is easy (open-source harnesses are as good as Claude Code/Codex), but owning the harness means owning the token bill, and asking users to pay for tokens in a new system kills adoption. Ride existing agents through the three phases — copy-paste, MCP, auto-run — and revisit only if a reason appears.
- **No native apps as an output target.** Web only.
- **No solving human-edit version handling yet.** Git-backed editing of rendered views is a real problem, parked.
- **No confirmation dialogs in-app yet.** "Is this why you hated it?" loops will initially surface in the agent's chat; the in-viewer confirmation UI comes later.
- **No global/cross-project memory yet.** Boxydoc is per-repo. A global layer of reusable context ("import UI preferences") is easy to add later; premature now.
