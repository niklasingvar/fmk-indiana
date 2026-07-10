---
status: draft
purpose: Push to the indiana-loop — annotatable HTML preview in Casablanca + context-model loop wired and demonstrated end to end.
---

# Plan — the indiana-loop push

## Context

FOCUS.md declares the target: a user makes commands in Casablanca — both inline in rich markdown and by annotating rendered HTML — clicks `Copy all`, pastes into any coding agent, and three things happen: (1) the artifact is updated, (2) `.indiana/context-model/` is updated per the schema, (3) `.indiana/chief-of-staff/` captures tasks. VISION.md and ACTION_PLAN Phase 6 already promise "annotation boxes emit ordinary `::` markers into source."

What exists: Casablanca edits markdown with byte-stable `::` round-trip and a working `Copy all` (shells `indiana copy`); the compiled payload opens with the INDIANA LOOP preamble (read context-model → write back `log.md` + `focus.md`); `::hate`/`::love`/`::note` prompts instruct INBOX write-back. What's missing: HTML files are invisible to the editor (tree/watcher filter `.md` only), there is no preview or annotation surface, the dogfood context-model is pristine seeds (the loop has never run), and ITERATION-GUIDE.md is an empty stub.

Decisions taken with the operator (2026-07-06):
- **Annotations land in a sidecar markdown file** next to the HTML — zero Rust changes; the `.md`-only scanner invariant stands; markers stay human-editable markdown that `indiana copy` already picks up (`page.html.md` has extension `md`).
- **Adopt `files/context-model/`** (the prepared, populated tree) as this repo's dogfood `.indiana/context-model/`.

Engine facts that shape the design (verified in `crates/core`):
- Inline markers (non-whitespace before `::` on the line) get `ScopeKind::Inline`: `scope_content` = the text before the marker (scope.rs:41-44). This carries the annotation target — **even for reactions** (`::hate` drops its trailing message but scope is computed independently).
- Column-0 markers scope *forward* (next block), so a marker-on-own-line format would attach to the wrong content. One-line inline format it is.
- Parser hazards: a second `::token` on a line → whole line skipped as Ambiguous; backticks open code spans that swallow markers. Generated lines must be sanitized.

## Workstream A — HTML browser + annotation bubbles in Casablanca

### Architecture
- **Preview**: `<iframe>` as a normal flex child of EditorPane, backed by a custom `vault://` protocol (`protocol.handle` in main, scheme registered privileged/`standard:true` before `app.whenReady` so relative css/js/img in vault HTML resolve through the same handler). Path-traversal guard resolves against the current vault root. `Cache-Control: no-store`. CSP in `src/renderer/index.html` gains `frame-src vault:`.
- **Annotator injection**: for `text/html` responses the handler injects `<script src="/__casablanca__/annotator.js">` (served same-origin from an app-bundled `?raw` string — survives vault pages with `script-src 'self'` CSPs). The iframe is sandboxed (`allow-scripts allow-same-origin allow-forms`), has no preload and no `window.api`; its only channel out is `postMessage` to the host.
- **Annotator script** (plain JS, ~120 lines, sensing only): hover outline via inline style; click (capture, preventDefault when annotate-mode is on — toolbar toggle, default on) posts `{docRelPath, selector, excerpt, rect}`; scroll/resize posts invalidate (host closes bubble). Selector: `#id` if unique, else `tag:nth-of-type(k)` path, single-colon pseudos only. Excerpt: innerText/alt/aria-label, whitespace-collapsed, 80 chars.
- **Bubble**: rendered by React/Tailwind in the host over the iframe at the posted rect. Nine commands — question, fix, elaborate, hate, love, keep, delete, note, todo — honoring each kind's message contract from the marker TABLE: hate/love/keep no text input (one click), note/todo required text, rest optional.
- **Sidecar**: `<doc>.html.md` beside the HTML. Created with header `# Annotations — <doc>.html`; each annotation appends one line:

  ```
  - [site/page.html] main > section:nth-of-type(2) > h2 — "Pricing tiers" ::fix align the columns
  ```

  Compiled payload block the agent then sees:

  ```
  <vault>/site/page.html.md:5 [fix]
  Fix this. align the columns

  - [site/page.html] main > section:nth-of-type(2) > h2 — "Pricing tiers"
  ```

  Sanitizer (shared, tested): collapse whitespace/newlines, strip backticks, collapse `::+` → `:` in selector/excerpt/message, `"` → `'` in excerpt, truncate excerpt.
- **Tree/watcher/loop payoff**: `vault.ts` walks `.md|.html|.htm` (new `isTracked`; `isNote` stays `.md`-only so New-note never creates HTML). `watcher.ts` adds html globs and pushes `preview:changed` (posix rel path, debounced) → HtmlPreview bumps a cache-busting query and reloads — *agent edits HTML, preview refreshes*. `FileTree.stripMd` keeps `.md` visible when the stem ends `.html` so `page.html` and `page.html.md` stay distinct rows.
- **IPC** (existing quadruple pattern — `shared/ipc.ts`, `shared/domain.ts`, `main/ipc.ts`, `preload/index.ts`): `annotation:append` (main builds the line, plain fs create/append — no marker parsing; "core computes, faces render") and `preview:changed`. `registerIpc` returns a `getVault` accessor so the protocol handler sees vault switches.
- **useVault**: `openNote` branches on `.html` — stub activeNote, `draft = null` (no Lexical, no autosave); EditorPane renders three-way: HtmlPreview | Lexical | empty.

### Steps (each independently landable)
1. `src/shared/annotation-line.ts` (+test): `ANNOTATION_KINDS` table (token + message contract, mirrors markers.rs), `sanitizeInline`, `buildAnnotationLine`, `sidecarHeader`, `isHtmlPath`. The engine-compatibility linchpin.
2. `src/shared/ipc.ts` + `src/shared/domain.ts`: channels + `AnnotationKind`/`AnnotationRequest`/`AnnotationResult` types.
3. `src/main/preview/`: `resolve-path.ts` (+test — traversal incl. `%2e%2e`, mime map), `annotator.js`, `protocol.ts` (handler + injection; `raw.d.ts` for `?raw`).
4. Main wiring: `src/main/index.ts` (privileged scheme, register protocol), `src/main/ipc.ts` (append handler + `getVault`), `src/main/lib/annotations.ts` (+test — create-with-header vs append, trailing-newline repair, reaction/required contracts).
5. `src/main/lib/vault.ts` + `src/main/watcher.ts`: tracked extensions, `preview:changed`.
6. `src/preload/index.ts`: `annotations.append`, `preview.onChanged`.
7. Renderer: `src/renderer/src/preview/HtmlPreview.tsx` + `AnnotationBubble.tsx` (new), `EditorPane.tsx` branch, `useVault.ts` html path, `FileTree.tsx` stripMd tweak (3 lines — file has uncommitted Track-3 work, touch minimally), `index.html` CSP.
8. Typecheck, tests, manual verification (below).

### Edge cases handled / accepted
- Sanitizer prevents Ambiguous lines (`::` in message) and code-span swallowing (backticks).
- In-iframe navigation to another vault page: annotator derives `docRelPath` from `location.pathname`; main re-validates.
- `::todo` gets `[id]` injected into the sidecar by the engine on next copy — byte-preserving, nothing in Casablanca parses it (verified compatible).
- Accepted for MVP: sidecar open in Lexical while appending can clobber (same hazard as engine ID-injection today; document "close/reopen"); pages with `script-src 'none'` or `<base href>` break annotation/assets; bubble closes on scroll instead of tracking.

## Workstream B — wire and demonstrate the loop

1. **Adopt the prepared context-model as dogfood.** Copy `files/context-model/` → `.indiana/context-model/` (full 213-line schema replaces the compressed seed — dogfood may diverge freely per MENTAL_MODEL). Fold the 5 hand-written lines of `.indiana/context-model/INDIANA.md` into `learnings/INBOX.md` as one dated entry, delete `INDIANA.md` and `.gitkeep`, append an adoption line to `log.md`.
2. **Clean the rename leftover.** `.indiana/montmartre/` is empty seed headers duplicating `.indiana/chief-of-staff/` — delete it.
3. **Sync audit: commands ↔ context-model.** Verify each prompt template's write-back against schema §7 (hate/love/note → INBOX ✓, todo → `chief-of-staff/focus.md` ✓, preamble → `log.md` ✓). Known gap to record in `architecture/DECISIONS.md` + INBOX: schema §9 says "lint runs via `::lint`" but no `::lint` marker exists in the TABLE — promotion candidate / fast-follow (the `create-indiana-command` skill automates adding it); not in this push.
4. **Write ITERATION-GUIDE.md for real.** Fill the stub's two sections: *Update context model* (authoring source is `files/CONTEXT-MODEL.md` → recompress into `crates/core/templates/context-model/CONTEXT-MODEL.md` seed → dogfood instance adopts directly; lint/consolidation cadence per schema §9) and *Update command templates* (edit `crates/core/templates/indianas/<cmd>/prompt.md`, pinned by `test_embedded_templates_match_marker_table`; dogfood `.indiana/indianas/` refresh is explicit, never automatic). Plus the loop demo script (below) so "how to loop things" is showable to anyone.
5. **Run the loop once, for real (the demo).** Open this repo in Casablanca → annotate a demo HTML file (`::fix` + `::hate` + `::todo`) and tag one inline `::note` in a markdown doc → `Copy all` → paste into Claude Code → verify all three outcomes: artifact edited, `log.md` + `learnings/INBOX.md` appended, `focus.md` gained the todo. This is ACTION_PLAN Phase 1's exit plus Phase 5's exit criterion.

## Execution order & commits
0. Land the current working-tree WIP first as its own commits (Track-2 tables + Track-3 tree polish) so this push's commits stay focused.
1. Workstream A steps 1–8 (small commits per step, per AGENT_COMMIT).
2. Workstream B steps 1–4.
3. B5 — the live demo — runs last, against the built editor.

## Out of scope (explicit)
- No server/daemon changes: Casablanca keeps shelling `indiana copy`; socket/MCP integration and the pending-marker badge stay Phase 2/3.
- No Rust engine changes: scanner stays `.md`-only; no `::lint` marker yet; no `::action`/`::prompt` in the bubble.
- No display/editing of existing annotations in the preview (the sidecar is an ordinary visible md file).
- No remote URLs in the preview — vault-local HTML only.

## Verification
- **Units** (`npm test` in `crates/casablanca`, vitest/node): annotation-line builder (contracts per kind, sanitization), annotations fs append/create, resolve-path traversal — plus the existing round-trip suites stay green. `npm run typecheck`.
- **Manual end-to-end** (fixture vault `/tmp/casa-vault/site/page.html` + `style.css`): tree shows the html → click renders it with its CSS (proves `vault://` relative assets) → hover outlines, click h2 → bubble → `fix` + message creates `page.html.md` with the exact line format → `hate` appends bare-reaction line → `note` with empty text is blocked → `echo >> page.html` reloads preview ≤1s → `Copy all` payload shows the fix block with prompt + scope line, and the hate block still carrying its target as scope → second copy injects `::todo[id]` byte-stably → tree shows `page.html` and `page.html.md` as distinct rows.
- **Loop demo** (Workstream B5): the three FOCUS.md outcomes observed in this repo's `.indiana/` after one paste.
