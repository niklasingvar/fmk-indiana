import type { NoteDocument } from './domain'

/**
 * Pure split/join between a raw markdown file and a NoteDocument.
 *
 * Parsing only ever splits the string and serializing only ever concatenates,
 * so `serializeNoteDocument(parseNoteDocument(raw)) === raw` holds for every
 * input — the editor cannot corrupt what it does not model.
 *
 * Fence rules mirror the Indiana engine (crates/core/src/templates.rs):
 * frontmatter exists only when the file starts with `---\n` and a closing
 * line that is exactly `---` follows. An unclosed fence is body, and a `---`
 * later in the file is an ordinary thematic break. One deliberate divergence:
 * the empty block `---\n---\n` counts as frontmatter here (the engine treats
 * it as unclosed); either reading serializes back to the same bytes.
 */

const OPEN = '---\n'
const CLOSE = '\n---\n'

export function parseNoteDocument(raw: string): NoteDocument {
  if (!raw.startsWith(OPEN)) return { frontmatter: null, body: raw }
  // Start the search on the opening fence's own newline so `---\n---\n`
  // (empty frontmatter) closes on the very next line.
  const close = raw.indexOf(CLOSE, OPEN.length - 1)
  if (close === -1) return { frontmatter: null, body: raw }
  const end = close + CLOSE.length
  return { frontmatter: raw.slice(0, end), body: raw.slice(end) }
}

export function serializeNoteDocument(doc: NoteDocument): string {
  return (doc.frontmatter ?? '') + doc.body
}
