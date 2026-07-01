AI-Native Workspace with learning loops and opinionated LLM friendly architecture

Working codenames: 
Indiana · headless command engine and CLI
Casablanca · editor and visual layer
Montmartre · project management - ADHD style
Boxydoc · context model and memory
Bangalore · app menulet

The Problem
The current way of working with AI on real artifacts — presentations, documents, code, apps — is broken in two directions.
The chat direction is exhausting: you describe what you want, read a wall of markdown, copy-paste it somewhere else, then come back to the chat and try to explain where in the artifact your feedback applies ("the third paragraph on the second slide, no not that one..."). Pointing at things through prose is cumbersome, and every round trip through chat is an expensive, slow feedback loop.
The agent direction isn't better: plan, run the agent, read a review dump, give feedback, repeat. The review artifacts are painful to read and the loop is just as long.
Meanwhile, the tooling itself has terrible UX. Power users end up writing prose documents inside code editors because that's where their tools live. Non-developers get chat interfaces with no ability to see or touch the actual artifact. Nobody gets to stay close to the thing they're actually making.
The core insight: the content is the only thing that matters. Everything else — the prompting, the loops, the reviewing — is overhead. The mission is to collapse that overhead so the human can stay focused on the artifact itself.
The Thesis
Stay closer to the artifact. Ideally, remove chat from the system entirely.
Instead of describing feedback in a chat window, you annotate the artifact directly — in the file, or on a rendered view of it — using a lightweight, fully configurable inline command language. An agent picks up those annotations, acts on them in the background while you keep working, and the artifact updates in place.
The unit of work is a folder. A folder is a mission: it holds the artifact, the context, and the configuration for how AI should behave in that project. This is deliberately aligned with where the world is going — every serious coding agent already operates against a folder. We don't fight that; we build on it.

The Dream
The dream is that I, as an operator, can iterate through my artifacts without entering the chat. Each time the Indiana Prom templates in.Indiana are aligned with BoxyDoc, which is the rule set of how we want things to be created. Those templates direct the code agents to pick up what to read and also update the metadata, meaning the context model, so that we keep all the rules and docs in.Indiana for this repo continuously. We have a strict domain tree information tree cone, and the agents continuously work towards creating this hierarchical information system.So we call this dream Compound Knowledge Through Loops, which is the absolute unique selling point for this type of workflow, besides the domain-specific language and the no-chat workspace.

The Components
The system is deliberately modular — four separable pieces that are valuable alone and compound together. Separating them means each can be built, tested, and killed independently.
Indiana — the command engine (the logic layer)
A domain-specific language for inline feedback, plus the server that acts on it.
You write commands directly in your files: ::fix, ::hate, ::love, ::note, or anything you define yourself. Each command maps to an instruction set that tells the LLM what mode to enter. ::hate doesn't just mean "change this" — it means "diagnose why the user hates this, explain it back, and update the project's context model so it never happens again."
Everything lives in a .indiana/ folder inside the repository:
commands/ — one file per command (fix.md, design.md, amend.md...). Fully configurable per project, so ::design means one thing in a dev repo and another in a presentation project.
context/ — the project's persistent context model: purpose, preferences, accumulated learnings from past feedback. Every command execution reads it; commands like ::hate write back to it. This is how the system stops repeating mistakes.
A CLI collects annotations across the repo (indiana copy), and a lightweight server can continuously monitor the folder so annotations are picked up and executed in the background — you mark ::fix on a line, keep reading, and it's fixed by the time you scroll back.
Crucially, Indiana is harness-agnostic. In the first version it doesn't run its own agent at all — you paste the collected commands into Claude Code, Codex, Cursor, whatever you already use. Their tokens, their quota, their harness. Indiana operates against the folder; so do they. It complements them rather than competing with them, which sidesteps the fatal problem of asking users to buy tokens in a new system.
Casablanca — editor and the viewer (the visual layer)
Cursor is a terrible place to look at a presentation. Chat is a terrible place to point at slide three. Casablanca renders artifacts as what they actually are — slides as slides, documents as documents — with annotation directly on the rendered view. Mark a section as hate, click run, and the feedback flows into Indiana.
Artifact types, roughly in order of attack:
Presentations — rendered slide decks with annotation boxes.
Documents / text — a proper reading and annotating view for markdown.
Code —(devs want raw).
Excalidraw - lets be visual
Apps — web only, always. The DOM gives the system a way to know what happened to the UI; native doesn't.
The spectrum Casablanca navigates: raw/native mode (maximum speed and flexibility, crap UX) versus rendered mode (great UX, but human edits require version handling — a real technical problem, parked for now).
Note: Inspired by nimbalyst a open source repo
Note: local settings.json is nice
Montmartre — project management - ADHD style
It stores Human TODO and Agent TODO in a local sqlite
Indiana picks them up
Bangalore — menulet

For showing what focus is, project status, and a quick copy paste thing


---
How It Works: The Presentation Flow
You as operator select a repository (folder)
You create a md-file (in casablanca) brief.md 
You type what you want “a five slides pitching X."
Laziness kicks in and you do ::elaborate
You click run and a reversed MCP makes claude code run the “copied payload”
Because of prompt template a elaborated brief.md
You hit ::prompt render the slides
CodeAgent (whichever) renders the deck. Casablanca helps you vizulize it, You annotate directly on it: ::hate on a couple of things, ::fix on the title, a free-text note on slide four.
You hit run. Indiana diagnoses the feedback, updates content files (cheap, fast) without touching design files, logs the change the important context files, and updates the context model so the next iteration already knows your preferences
Repeat — but each loop is shorter, and there are fewer of them.
The template-first, content/design-separated architecture is the key economic move: rough drafts are token-cheap (copy a template, fill it in), content iterations don't rewrite design, and design iterations don't rewrite content. Per-change cost may be higher than raw markdown chat, but the promise is fewer total loops from nothing to good enough.
What This Becomes
Individually, these are tools. Together — a folder as mission, context that accumulates, inline commands, background execution, a viewer, a focus layer — this is a work management system for the AI-native era. Not project management as status-tracking, but the actual environment where work happens: you pick a folder, the mission and context live in it, any AI can connect to it, and the feedback loop between you and the artifact is as short as writing two colons.
Who It's For
Now: me. One user, power-user mode, dogfooded on real projects (starting with the card game). Developers are the natural first audience — the annotate-specific-lines problem is already solved for them and they'll feel the speed immediately.
Later: the non-dev who's ready for agentic workflows — the manager writing a pitch deck, the person who wants to see and touch their document. Bangalore and buttons are the bridge. Same system underneath; the config files are just invisible to them.
Deliberate Non-Goals (For Now)
No own harness. Building one is easy (open-source harnesses are as good as Claude Code/Codex), but owning the harness means owning the token bill, and asking users to pay for tokens in a new system kills adoption. Ride existing agents until there's a reason not to.
No native apps as an output target. Web only.
No solving human-edit version handling yet. Git-backed editing of rendered views is a real problem, parked.
No confirmation dialogs in-app yet. "Is this why you hated it?" loops will initially surface in the agent's chat; the in-viewer confirmation UI comes later.
