import { describe, expect, it } from 'vitest'
import { parsePorcelain } from './git'

describe('parsePorcelain', () => {
  it('maps untracked, modified, staged, and deleted files', () => {
    const map = parsePorcelain(
      ['?? docs/new.md', ' M src/app.ts', 'A  fresh.md', ' D gone.md', 'MM both.md', ''].join('\n')
    )
    expect(map['docs/new.md']).toBe('new')
    expect(map['src/app.ts']).toBe('modified')
    expect(map['fresh.md']).toBe('new')
    expect(map['gone.md']).toBe('deleted')
    expect(map['both.md']).toBe('modified')
  })

  it('aggregates folder status with modified winning over new', () => {
    const map = parsePorcelain(['?? plans/a.md', ' M plans/b.md', '?? assets/logo.png'].join('\n'))
    expect(map['plans']).toBe('modified')
    expect(map['assets']).toBe('new')
  })

  it('takes the new path of a rename and unquotes odd names', () => {
    const map = parsePorcelain(['R  old.md -> docs/renamed.md', '?? "my file.md"'].join('\n'))
    expect(map['docs/renamed.md']).toBe('modified')
    expect(map['docs']).toBe('modified')
    expect(map['my file.md']).toBe('new')
  })

  it('returns an empty map for empty output', () => {
    expect(parsePorcelain('')).toEqual({})
  })
})
