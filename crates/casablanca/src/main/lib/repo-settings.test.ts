import { describe, expect, it } from 'vitest'
import { parseRepoSettings, repoColorOf } from './repo-settings'

describe('parseRepoSettings', () => {
  it('reads a JSON object', () => {
    expect(parseRepoSettings('{"color":"1 2 3","wrap":true}')).toEqual({
      color: '1 2 3',
      wrap: true
    })
  })

  it('degrades non-object / invalid JSON to an empty bag', () => {
    expect(parseRepoSettings('[1,2,3]')).toEqual({})
    expect(parseRepoSettings('not json')).toEqual({})
    expect(parseRepoSettings('null')).toEqual({})
  })
})

describe('repoColorOf', () => {
  it('returns a non-empty string color', () => {
    expect(repoColorOf({ color: '255 90 20' })).toBe('255 90 20')
  })

  it('ignores a missing, empty, or non-string color', () => {
    expect(repoColorOf({})).toBeNull()
    expect(repoColorOf({ color: '   ' })).toBeNull()
    expect(repoColorOf({ color: 42 })).toBeNull()
  })
})
