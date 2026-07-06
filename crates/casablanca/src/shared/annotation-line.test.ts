import { describe, expect, it } from 'vitest'
import {
  ANNOTATION_KINDS,
  buildAnnotationLine,
  isHtmlPath,
  sanitizeInline,
  sidecarHeader,
  sidecarPath
} from './annotation-line'
import type { AnnotationRequest } from './domain'

const base: AnnotationRequest = {
  docRelPath: 'site/page.html',
  selector: 'main > section:nth-of-type(2) > h2',
  excerpt: 'Pricing tiers',
  kind: 'fix',
  message: 'align the columns'
}

describe('buildAnnotationLine', () => {
  it('builds the inline form: target before the marker, message after', () => {
    expect(buildAnnotationLine(base)).toBe(
      '- [site/page.html] main > section:nth-of-type(2) > h2 — "Pricing tiers" ::fix align the columns'
    )
  })

  it('drops the message for reactions but keeps the target as scope', () => {
    expect(buildAnnotationLine({ ...base, kind: 'hate', message: 'ignored' })).toBe(
      '- [site/page.html] main > section:nth-of-type(2) > h2 — "Pricing tiers" ::hate'
    )
  })

  it('throws when a required message is missing or blank', () => {
    expect(() => buildAnnotationLine({ ...base, kind: 'todo', message: '' })).toThrow(/requires/)
    expect(() => buildAnnotationLine({ ...base, kind: 'note', message: '   ' })).toThrow(/requires/)
  })

  it('omits the quoted excerpt segment when the excerpt is empty', () => {
    expect(buildAnnotationLine({ ...base, excerpt: '', kind: 'love' })).toBe(
      '- [site/page.html] main > section:nth-of-type(2) > h2 ::love'
    )
  })

  it('emits exactly one :: even with hostile input in every field', () => {
    const line = buildAnnotationLine({
      docRelPath: 'a::b/page.html',
      selector: 'div::before > `code`',
      excerpt: 'said "use ::fix here"\nand `more`',
      kind: 'note',
      message: 'try ::todo maybe\n\twith `ticks`'
    })
    expect(line.match(/::/g)).toHaveLength(1)
    expect(line).not.toContain('`')
    expect(line).not.toContain('\n')
    expect(line).toContain('::note ')
  })

  it('truncates the excerpt to 80 chars and swaps double quotes for singles', () => {
    const line = buildAnnotationLine({ ...base, excerpt: `he said "hi" ${'x'.repeat(200)}` })
    const quoted = line.match(/— "([^"]*)"/)
    expect(quoted).not.toBeNull()
    expect(quoted![1].length).toBeLessThanOrEqual(80)
    expect(quoted![1]).toContain("'hi'")
  })
})

describe('sanitizeInline', () => {
  it('collapses whitespace runs and newlines to single spaces', () => {
    expect(sanitizeInline('a\n\n b\t\tc')).toBe('a b c')
  })

  it('collapses :: runs so a fragment can never form a second marker', () => {
    expect(sanitizeInline('x :::: y :: z')).toBe('x : y : z')
  })
})

describe('kind table', () => {
  it('covers the nine bubble commands with the marker-table contracts', () => {
    const contracts = Object.fromEntries(ANNOTATION_KINDS.map((s) => [s.kind, s.message]))
    expect(contracts).toEqual({
      question: 'optional',
      fix: 'optional',
      elaborate: 'optional',
      hate: 'none',
      love: 'none',
      keep: 'none',
      delete: 'optional',
      note: 'required',
      todo: 'required'
    })
  })
})

describe('paths', () => {
  it('detects html paths case-insensitively', () => {
    expect(isHtmlPath('a/b.html')).toBe(true)
    expect(isHtmlPath('a/b.HTM')).toBe(true)
    expect(isHtmlPath('a/b.html.md')).toBe(false)
    expect(isHtmlPath('a/b.md')).toBe(false)
  })

  it('derives the sidecar path and header', () => {
    expect(sidecarPath('site/page.html')).toBe('site/page.html.md')
    expect(sidecarHeader('site/page.html')).toBe('# Annotations — site/page.html\n')
  })
})
