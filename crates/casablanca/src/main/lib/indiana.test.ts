import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { listAgents, resolveIndianaBinary, toVaultMarkers } from './indiana'

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

describe('toVaultMarkers', () => {
  it('relativizes paths and maps snake_case fields', () => {
    const scanJson = JSON.stringify({
      markers: [
        {
          path: '/vault/notes/plan.md',
          line: 12,
          column: 4,
          kind: 'fix',
          raw_token: '::f',
          message: 'tighten this',
          id: 'happy-otter',
          status: 'working',
          scope: { kind: 'inline', content: 'x' }
        },
        {
          path: '/elsewhere/loose.md',
          line: 1,
          column: 1,
          kind: 'note',
          raw_token: '::note',
          scope: { kind: 'inline', content: 'y' }
        }
      ]
    })

    const markers = toVaultMarkers(scanJson, '/vault')

    expect(markers).toEqual([
      {
        path: 'notes/plan.md',
        line: 12,
        kind: 'fix',
        rawToken: '::f',
        message: 'tighten this',
        group: undefined,
        agent: undefined,
        id: 'happy-otter',
        status: 'working'
      },
      {
        path: '/elsewhere/loose.md',
        line: 1,
        kind: 'note',
        rawToken: '::note',
        message: undefined,
        group: undefined,
        agent: undefined,
        id: undefined,
        status: undefined
      }
    ])
  })

  it('handles a root that already ends with a slash', () => {
    const scanJson = JSON.stringify({
      markers: [
        { path: '/vault/a.md', line: 3, column: 1, kind: 'todo', raw_token: '::todo' }
      ]
    })

    expect(toVaultMarkers(scanJson, '/vault/')[0].path).toBe('a.md')
  })
})

describe('toVaultMarkers', () => {
  it('carries numeric group and agent persona through to the renderer shape', () => {
    const scanJson = JSON.stringify({
      markers: [
        {
          path: '/vault/doc.md',
          line: 3,
          kind: 'fix',
          raw_token: '::fix',
          message: 'create this task',
          agent: 'mike'
        },
        {
          path: '/vault/doc.md',
          line: 7,
          kind: 'note',
          raw_token: '::note',
          message: 'batch it',
          group: 2
        }
      ]
    })

    const markers = toVaultMarkers(scanJson, '/vault')
    expect(markers[0]).toMatchObject({ path: 'doc.md', agent: 'mike', group: undefined })
    expect(markers[1]).toMatchObject({ path: 'doc.md', group: 2, agent: undefined })
  })
})

describe('listAgents', () => {
  it('lists directories under .indiana/agents that carry a SYSTEM_PROMPT.md', async () => {
    for (const name of ['mike', 'lisa']) {
      const dir = join(fixtureDir, '.indiana', 'agents', name)
      mkdirSync(dir, { recursive: true })
      writeFileSync(join(dir, 'SYSTEM_PROMPT.md'), 'prompt')
    }
    // Missing prompt file and flag-unsafe names are not agents.
    mkdirSync(join(fixtureDir, '.indiana', 'agents', 'empty'), { recursive: true })
    const bad = join(fixtureDir, '.indiana', 'agents', '1nope')
    mkdirSync(bad, { recursive: true })
    writeFileSync(join(bad, 'SYSTEM_PROMPT.md'), 'prompt')

    const result = await listAgents({ rootPath: fixtureDir })
    expect(result.agents).toEqual(['lisa', 'mike'])
  })

  it('reads an absent agents directory as an empty roster', async () => {
    const result = await listAgents({ rootPath: fixtureDir })
    expect(result.agents).toEqual([])
  })
})
