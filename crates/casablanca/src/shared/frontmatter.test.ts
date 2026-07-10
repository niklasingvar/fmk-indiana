import { describe, expect, it } from 'vitest'
import {
  addFrontmatterAnnotation,
  frontmatterAnnotations,
  frontmatterSource,
  normalizeCommandText,
  projectFrontmatter,
  removeFrontmatterProperty,
  setFrontmatterProperty,
  wrapFrontmatter
} from './frontmatter'

describe('frontmatter source', () => {
  it('unwraps and wraps the fences without exposing them to the editor', () => {
    expect(frontmatterSource('---\nstatus: draft\n---\n')).toBe('status: draft\n')
    expect(wrapFrontmatter('status: draft')).toBe('---\nstatus: draft\n---\n')
  })
})

describe('property projection', () => {
  it('projects top-level scalar values', () => {
    expect(projectFrontmatter('---\nstatus: draft\nmax_lines: 70\npublic: false\nowner:\n---\n')).toEqual({
      kind: 'properties',
      properties: [
        { key: 'status', value: 'draft' },
        { key: 'max_lines', value: 70 },
        { key: 'public', value: false },
        { key: 'owner', value: null }
      ]
    })
  })

  it('falls back to raw YAML for nested or malformed content', () => {
    expect(projectFrontmatter('---\ntags:\n  - one\n---\n').kind).toBe('raw')
    expect(projectFrontmatter('---\nstatus: [\n---\n').kind).toBe('raw')
  })
})

describe('property edits', () => {
  const block = '---\nstatus: draft\n# frontmatter.status ::fix approve it\npurpose: Test\n---\n'

  it('sets and adds properties while preserving comments', () => {
    const changed = setFrontmatterProperty(block, 'status', 'approved')
    expect(changed).toContain('status: approved')
    expect(changed).toContain('# frontmatter.status ::fix approve it')

    const added = setFrontmatterProperty(changed, 'approval', 'pending')
    expect(projectFrontmatter(added)).toMatchObject({
      kind: 'properties',
      properties: expect.arrayContaining([{ key: 'approval', value: 'pending' }])
    })

    expect(setFrontmatterProperty('---\n---\n', 'status', 'draft')).toBe(
      '---\nstatus: draft\n---\n'
    )
  })

  it('removes a property', () => {
    const changed = removeFrontmatterProperty(block, 'purpose')
    expect(changed).not.toContain('purpose:')

    const annotated = removeFrontmatterProperty(block, 'status')
    expect(annotated).not.toContain('status:')
    expect(annotated).not.toContain('# frontmatter.status')
  })
})

describe('property annotations', () => {
  it('inserts an explicit YAML comment beside the property', () => {
    const block = '---\nstatus: draft\npurpose: Test\n---\n'
    const changed = addFrontmatterAnnotation(block, 'status', '  ::fix   change to approved  ')
    expect(changed).toBe(
      '---\nstatus: draft\n# frontmatter.status ::fix change to approved\npurpose: Test\n---\n'
    )
    expect(frontmatterAnnotations(changed, 'status')).toEqual(['::fix change to approved'])
  })

  it('encodes unusual property names in the comment target', () => {
    const changed = addFrontmatterAnnotation('---\n"review state": draft\n---\n', 'review state', '::question')
    expect(changed).toContain('# frontmatter.review%20state ::question')
    expect(frontmatterAnnotations(changed, 'review state')).toEqual(['::question'])
  })

  it('validates one visible command without restricting its token', () => {
    expect(normalizeCommandText(' ::custom   do this ')).toBe('::custom do this')
    expect(() => normalizeCommandText('fix it')).toThrow(/Start/)
    expect(() => normalizeCommandText('::fix then ::todo later')).toThrow(/one/)
    expect(() => normalizeCommandText('::fix `sample`')).toThrow(/Backticks/)
  })
})
