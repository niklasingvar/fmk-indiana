import { describe, expect, it } from 'vitest'
import { parseUnifiedDiff } from './diff'

const PATCH = [
  'diff --git a/notes/todo.md b/notes/todo.md',
  'index 1234567..89abcde 100644',
  '--- a/notes/todo.md',
  '+++ b/notes/todo.md',
  '@@ -1,3 +1,3 @@',
  ' # Todo',
  '-buy milk',
  '+buy oat milk',
  ' walk the dog',
  ''
].join('\n')

describe('parseUnifiedDiff', () => {
  it('drops file headers and types hunk/context/removed/added lines', () => {
    expect(parseUnifiedDiff(PATCH)).toEqual([
      { kind: 'hunk', text: '@@ -1,3 +1,3 @@' },
      { kind: 'context', text: '# Todo' },
      { kind: 'removed', text: 'buy milk' },
      { kind: 'added', text: 'buy oat milk' },
      { kind: 'context', text: 'walk the dog' }
    ])
  })

  it('does not mistake ---/+++ headers for removed/added lines', () => {
    const kinds = parseUnifiedDiff(PATCH).map((l) => l.kind)
    expect(kinds.filter((k) => k === 'removed')).toHaveLength(1)
    expect(kinds.filter((k) => k === 'added')).toHaveLength(1)
  })

  it('skips the no-newline marker', () => {
    const patch = ['@@ -1 +1 @@', '-old', '+new', '\\ No newline at end of file', ''].join('\n')
    expect(parseUnifiedDiff(patch)).toEqual([
      { kind: 'hunk', text: '@@ -1 +1 @@' },
      { kind: 'removed', text: 'old' },
      { kind: 'added', text: 'new' }
    ])
  })

  it('handles an all-added diff for a new file', () => {
    const patch = [
      'diff --git a/dev/null b/fresh.md',
      'new file mode 100644',
      '--- /dev/null',
      '+++ b/fresh.md',
      '@@ -0,0 +1,2 @@',
      '+# Fresh',
      '+first line',
      ''
    ].join('\n')
    expect(parseUnifiedDiff(patch)).toEqual([
      { kind: 'hunk', text: '@@ -0,0 +1,2 @@' },
      { kind: 'added', text: '# Fresh' },
      { kind: 'added', text: 'first line' }
    ])
  })

  it('returns empty for empty input', () => {
    expect(parseUnifiedDiff('')).toEqual([])
  })
})
