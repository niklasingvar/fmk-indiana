// ::ignore
import { describe, expect, it } from 'vitest'
import { revealMatchRank } from './reveal-match'

describe('revealMatchRank', () => {
  it('ranks an exact line match highest', () => {
    expect(revealMatchRank('Ship the thing ::fix tighten', 'Ship the thing ::fix tighten')).toBe(3)
  })

  it('matches by the marker tail when markdown stripped the prefix', () => {
    // On disk: `- item ::fix message`; in the editor the listitem text is
    // `item ::fix message` — no bullet.
    expect(revealMatchRank('item ::fix message', '- item ::fix message')).toBe(2)
  })

  it('matches a claimed marker line whose bracket is present in both', () => {
    expect(
      revealMatchRank('intro ::fix[happy-otter:working] msg', '# intro ::fix[happy-otter:working] msg')
    ).toBe(2)
  })

  it('falls back to plain containment for lines without a marker token', () => {
    expect(revealMatchRank('prefix some plain line suffix', 'some plain line')).toBe(1)
  })

  it('does not match unrelated text or blank lines', () => {
    expect(revealMatchRank('nothing to see', 'other ::fix line')).toBe(0)
    expect(revealMatchRank('anything', '   ')).toBe(0)
  })
})
