import { describe, expect, it } from 'vitest'
import { parseNoteDocument, serializeNoteDocument } from './note-serialization'

const roundTrip = (raw: string): string => serializeNoteDocument(parseNoteDocument(raw))

describe('parseNoteDocument', () => {
  it('splits a normal frontmatter block from the body', () => {
    const raw = '---\nstatus: draft\npurpose: test\n---\n\n# Title\n\nBody.\n'
    const doc = parseNoteDocument(raw)
    expect(doc.frontmatter).toBe('---\nstatus: draft\npurpose: test\n---\n')
    expect(doc.body).toBe('\n# Title\n\nBody.\n')
  })

  it('returns null frontmatter when the file has none', () => {
    const raw = '# Title\n\nBody.\n'
    expect(parseNoteDocument(raw)).toEqual({ frontmatter: null, body: raw })
  })

  it('treats an empty frontmatter block as frontmatter', () => {
    const raw = '---\n---\nBody.\n'
    const doc = parseNoteDocument(raw)
    expect(doc.frontmatter).toBe('---\n---\n')
    expect(doc.body).toBe('Body.\n')
  })

  it('treats an unclosed fence as body', () => {
    const raw = '---\nstatus: draft\nno closing fence\n'
    expect(parseNoteDocument(raw)).toEqual({ frontmatter: null, body: raw })
  })

  it('does not treat a mid-file thematic break as a fence', () => {
    const raw = '# Title\n\n---\n\nBelow the break.\n'
    expect(parseNoteDocument(raw).frontmatter).toBeNull()
  })

  it('handles a file that is only frontmatter', () => {
    const raw = '---\nstatus: draft\n---\n'
    const doc = parseNoteDocument(raw)
    expect(doc.frontmatter).toBe(raw)
    expect(doc.body).toBe('')
  })

  it('does not close on --- embedded in a value, only on an exact --- line', () => {
    const raw = '---\ntitle: a --- b\nnote: "c---d"\n---\nBody.\n'
    const doc = parseNoteDocument(raw)
    expect(doc.frontmatter).toBe('---\ntitle: a --- b\nnote: "c---d"\n---\n')
    expect(doc.body).toBe('Body.\n')
  })

  it('closes on the first exact --- line, like the engine', () => {
    // A multiline YAML string containing a bare `---` line closes early —
    // documented engine-consistent behavior, and still byte-stable.
    const raw = '---\ndesc: |\n  first\n---\n  second\n---\nBody.\n'
    const doc = parseNoteDocument(raw)
    expect(doc.frontmatter).toBe('---\ndesc: |\n  first\n---\n')
    expect(roundTrip(raw)).toBe(raw)
  })
})

describe('round-trip byte stability', () => {
  const cases: Record<string, string> = {
    'plain body': '# Title\n\nBody with ::fix marker.\n',
    'normal frontmatter': '---\nstatus: draft\n---\n\nBody.\n',
    'empty frontmatter': '---\n---\nBody.\n',
    'unclosed fence': '---\nstatus: draft\n',
    'frontmatter only': '---\nstatus: draft\n---\n',
    'body starting with thematic break': '---\n\nnot frontmatter, a break\n',
    'empty file': '',
    'single newline': '\n',
    'no trailing newline': '---\nstatus: x\n---\nbody without newline',
    'crlf content untouched': '# A\r\n\r\nB\r\n'
  }
  for (const [name, raw] of Object.entries(cases)) {
    it(name, () => {
      expect(roundTrip(raw)).toBe(raw)
    })
  }
})
