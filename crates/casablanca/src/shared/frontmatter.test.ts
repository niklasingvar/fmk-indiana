import { describe, expect, it } from 'vitest'
import { frontmatterSource, wrapFrontmatter } from './frontmatter'

describe('frontmatter source', () => {
  it('unwraps and wraps the fences without exposing them to the editor', () => {
    expect(frontmatterSource('---\nstatus: draft\n---\n')).toBe('status: draft\n')
    expect(wrapFrontmatter('status: draft')).toBe('---\nstatus: draft\n---\n')
  })
})
