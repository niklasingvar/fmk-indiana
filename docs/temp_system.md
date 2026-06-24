# System: Indiana

You work through a coding agent. It emits a lot — code, plans, docs, slides. You review it all, every day, deciding what's right, what's wrong, what to keep, what to fix. That review step is the bottleneck: you're fast, the agent is fast, but the handoff between you is slow. You end up typing the same feedback paragraphs over and over.

Indiana closes that gap. It makes the review step near-invisible.

---

## How it works

When you review what the agent produced, you don't explain. You **tag**. A `::` at the end of a line is all it takes — `::hate`, `::love`, `::keep`, `::fix`, `::elaborate`, `::question`, `::note`, `::action`. One keystroke, no essay.

Indiana watches the repo. It scans every markdown file, finds every `::` marker, and compiles them into an agent-ready payload — each marker expanded into a full prompt, its surrounding content bundled as context, its file path and line number attached. The agent reads that payload directly through MCP. You never retype a thing.

That's the core loop: agent emits → you tag → Indiana compiles → agent acts.

---

## The pieces

**Indiana** is the server. It monitors repos, owns the marker grammar end to end, and compiles the payload. One daemon, one static binary. CLI, MCP, and a macOS menulet all talk to it over a local socket. Faces display; the core computes.

**Menulet** is the GUI. It lives in your menu bar, shows which folders Indiana watches, and lets you copy a folder's compiled bundle in one click. It never scans, never counts — it's just a window onto Indiana's work.

**Casablanca** is the other side of the loop: a terse, token-friendly template format for what the agent *emits*. Instead of verbose markdown, the agent outputs structured, review-first artifacts that a human can scan in seconds. Casablanca visualizes that output; Indiana monitors your reactions to it. They're independent products that share a workflow.

**Meta Model** is the methodology behind it all — the bigger loop. You don't just ship features; you update the specs, the architecture, the principles as you go. It's an improved ADLC cycle (Analyze → Design → Loop → Close) where the meta model itself evolves with every iteration. The project gets smarter, not just bigger.

---

## The promise

You tag. You copy. You paste. The agent acts. The friction between judgment and action disappears. What's left is your speed — and the agent's.
