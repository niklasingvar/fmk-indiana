import { describe, expect, it } from 'vitest'
import { parseLog, parsePorcelain } from './git'

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

describe('parseLog', () => {
  it('parses tab-separated hash, epoch seconds, and subject', () => {
    const entries = parseLog(
      ['abc123\t1751900000\tfix | notes/todo.md — tightened the intro', ''].join('\n')
    )
    expect(entries).toEqual([
      {
        hash: 'abc123',
        timestamp: 1751900000000,
        subject: 'fix | notes/todo.md — tightened the intro'
      }
    ])
  })

  it('keeps tabs inside the subject and skips malformed lines', () => {
    const entries = parseLog(['abc\t1\tweird\tsubject', 'not-a-log-line', ''].join('\n'))
    expect(entries).toHaveLength(1)
    expect(entries[0].subject).toBe('weird\tsubject')
  })

  it('returns empty for empty output', () => {
    expect(parseLog('')).toEqual([])
  })
})
