/**
 * Builds the ordinary `::` marker lines that HTML annotations append to a
 * sidecar markdown file (`page.html` → `page.html.md`).
 *
 * The line format is chosen for the indiana engine's grammar (crates/core):
 * an inline marker — non-whitespace text before `::` on the same line — gets
 * inline scope, so everything before the marker (doc path, selector, excerpt)
 * rides into the compiled payload as the marker's target. Reactions
 * (hate/love/keep) drop their trailing message but keep that scope.
 *
 * Casablanca never parses markers ("core computes, faces render"); this
 * module only emits lines the engine can read, sanitized so a stray `::`,
 * backtick, or newline cannot break the line grammar — a second `::token`
 * on a line makes the whole line ambiguous and silently dead, and backticks
 * open inline code spans that swallow markers.
 */

import type { AnnotationKind, AnnotationRequest } from './domain'

export type MessageContract = 'none' | 'optional' | 'required'

export interface AnnotationKindSpec {
  kind: AnnotationKind
  /** Long marker token, written as `::<token>`. */
  token: string
  message: MessageContract
  /** Button label in the annotation bubble. */
  label: string
}

/**
 * The bubble's command set — message contracts mirror the marker TABLE in
 * crates/core/src/markers.rs (`::action` and `::prompt` are intentionally
 * not offered in the MVP bubble).
 */
export const ANNOTATION_KINDS: readonly AnnotationKindSpec[] = [
  { kind: 'question', token: 'question', message: 'optional', label: 'Question' },
  { kind: 'fix', token: 'fix', message: 'optional', label: 'Fix' },
  { kind: 'elaborate', token: 'elaborate', message: 'optional', label: 'Elaborate' },
  { kind: 'hate', token: 'hate', message: 'none', label: 'Hate' },
  { kind: 'love', token: 'love', message: 'none', label: 'Love' },
  { kind: 'keep', token: 'keep', message: 'none', label: 'Keep' },
  { kind: 'delete', token: 'delete', message: 'optional', label: 'Delete' },
  { kind: 'note', token: 'note', message: 'required', label: 'Note' },
  { kind: 'todo', token: 'todo', message: 'required', label: 'Todo' }
]

export function kindSpec(kind: AnnotationKind): AnnotationKindSpec {
  const spec = ANNOTATION_KINDS.find((s) => s.kind === kind)
  if (!spec) throw new Error(`unknown annotation kind: ${kind}`)
  return spec
}

const EXCERPT_MAX = 80

/** Make a fragment safe to sit on a marker line: one line, no backticks, no `::`. */
export function sanitizeInline(text: string): string {
  return text
    .replace(/\s+/g, ' ')
    .replace(/`/g, '')
    .replace(/:{2,}/g, ':')
    .trim()
}

export function isHtmlPath(p: string): boolean {
  return /\.html?$/i.test(p)
}

/** The sidecar sits next to the document: `site/page.html` → `site/page.html.md`. */
export function sidecarPath(htmlRelPath: string): string {
  return `${htmlRelPath}.md`
}

/** First line written when the sidecar is created. */
export function sidecarHeader(htmlRelPath: string): string {
  return `# Annotations — ${sanitizeInline(htmlRelPath)}\n`
}

/**
 * One annotation, one line:
 * `- [site/page.html] main > h2 — "Pricing tiers" ::fix align the columns`
 */
export function buildAnnotationLine(req: AnnotationRequest): string {
  const spec = kindSpec(req.kind)
  const doc = sanitizeInline(req.docRelPath)
  const selector = sanitizeInline(req.selector)
  const excerpt = sanitizeInline(req.excerpt)
    .replace(/"/g, "'")
    .slice(0, EXCERPT_MAX)
    .trim()
  const message = spec.message === 'none' ? '' : sanitizeInline(req.message ?? '')
  if (spec.message === 'required' && message === '') {
    throw new Error(`::${spec.token} requires a message`)
  }
  const target = excerpt === '' ? `- [${doc}] ${selector}` : `- [${doc}] ${selector} — "${excerpt}"`
  return message === '' ? `${target} ::${spec.token}` : `${target} ::${spec.token} ${message}`
}
