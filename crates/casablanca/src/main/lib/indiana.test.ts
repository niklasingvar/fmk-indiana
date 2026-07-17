import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { resolveIndianaBinary } from './indiana'

let originalIndianaBin: string | undefined
let fixtureDir: string

beforeEach(() => {
  originalIndianaBin = process.env.INDIANA_BIN
  fixtureDir = mkdtempSync(join(tmpdir(), 'casablanca-indiana-'))
})

afterEach(() => {
  if (originalIndianaBin === undefined) delete process.env.INDIANA_BIN
  else process.env.INDIANA_BIN = originalIndianaBin
  rmSync(fixtureDir, { recursive: true, force: true })
})

describe('resolveIndianaBinary', () => {
  it('prefers the binary supplied by the development launcher', () => {
    const binary = join(fixtureDir, 'indiana')
    writeFileSync(binary, '')
    process.env.INDIANA_BIN = binary

    expect(resolveIndianaBinary()).toBe(binary)
  })
})
