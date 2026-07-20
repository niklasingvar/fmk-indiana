// ::ignore
import { describe, expect, it } from 'vitest'
import { diffMarkerClaims } from './marker-claim'

describe('diffMarkerClaims', () => {
  it('recognizes a fresh claim minting an id', () => {
    const before = 'intro paragraph\n::fix -a fix the typo\ntrailer\n'
    const after = 'intro paragraph\n::fix[happy-otter:working] -a fix the typo\ntrailer\n'
    expect(diffMarkerClaims(before, after)).toEqual([
      { find: '::fix -a fix the typo', replace: '::fix[happy-otter:working] -a fix the typo' }
    ])
  })

  it('recognizes a status change on an existing bracket (working → failed)', () => {
    const before = 'intro\n::elaborate[frata-nimta:working] -a expand this\n'
    const after = 'intro\n::elaborate[frata-nimta:failed] -a expand this\n'
    expect(diffMarkerClaims(before, after)).toEqual([
      {
        find: '::elaborate[frata-nimta:working] -a expand this',
        replace: '::elaborate[frata-nimta:failed] -a expand this'
      }
    ])
  })

  it('recognizes a group claim touching several lines', () => {
    const before = '::fix -1 first\nmiddle\n::elaborate -1 second\n'
    const after = '::fix[happy-otter:working] -1 first\nmiddle\n::elaborate[calm-heron:working] -1 second\n'
    expect(diffMarkerClaims(before, after)).toEqual([
      { find: '::fix -1 first', replace: '::fix[happy-otter:working] -1 first' },
      { find: '::elaborate -1 second', replace: '::elaborate[calm-heron:working] -1 second' }
    ])
  })

  it('keeps surrounding text on the marker line', () => {
    const before = 'some text ::fix -a banana\n'
    const after = 'some text ::fix[happy-otter:working] -a banana\n'
    expect(diffMarkerClaims(before, after)).toEqual([
      { find: 'some text ::fix -a banana', replace: 'some text ::fix[happy-otter:working] -a banana' }
    ])
  })

  it('rejects a content edit alongside the claim', () => {
    const before = 'intro\n::fix -a fix the typo\n'
    const after = 'intro changed\n::fix[happy-otter:working] -a fix the typo\n'
    expect(diffMarkerClaims(before, after)).toBeNull()
  })

  it('rejects a change to the marker message', () => {
    const before = '::fix -a fix the typo\n'
    const after = '::fix[happy-otter:working] -a fix the typos\n'
    expect(diffMarkerClaims(before, after)).toBeNull()
  })

  it('rejects line-count changes (agent resolution deletes lines)', () => {
    const before = 'intro\n::fix[happy-otter:working] -a fix\ntrailer\n'
    const after = 'intro\ntrailer\n'
    expect(diffMarkerClaims(before, after)).toBeNull()
  })

  it('returns null when nothing differs', () => {
    const body = 'intro\n::fix -a fix\n'
    expect(diffMarkerClaims(body, body)).toBeNull()
  })

  it('rejects an arbitrary bracket-less line change', () => {
    const before = 'plain line\n'
    const after = 'plain line edited\n'
    expect(diffMarkerClaims(before, after)).toBeNull()
  })
})
