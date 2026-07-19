import { promises as fs } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import {
  ensureRepoDefaults,
  parseRepoSettings,
  readRepoSettings,
  repoColorOf,
  repoThemeOf
} from './repo-settings'

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

describe('repoThemeOf', () => {
  it('accepts light and dark', () => {
    expect(repoThemeOf({ theme: 'light' })).toBe('light')
    expect(repoThemeOf({ theme: 'dark' })).toBe('dark')
  })

  it('ignores missing or invalid values', () => {
    expect(repoThemeOf({})).toBeNull()
    expect(repoThemeOf({ theme: 'auto' })).toBeNull()
    expect(repoThemeOf({ theme: 1 })).toBeNull()
  })
})

describe('ensureRepoDefaults', () => {
  let root: string
  beforeEach(async () => {
    root = await fs.mkdtemp(join(tmpdir(), 'cb-repo-'))
  })
  afterEach(async () => {
    await fs.rm(root, { recursive: true, force: true })
  })

  it('defaults autoRun on for a fresh repo', async () => {
    await ensureRepoDefaults(root)
    expect((await readRepoSettings(root)).autoRun).toBe(true)
  })

  it('does not clobber a deliberate autoRun:false', async () => {
    await fs.mkdir(join(root, '.indiana', 'casablanca'), { recursive: true })
    await fs.writeFile(join(root, '.indiana', 'casablanca', 'settings.json'), '{"autoRun":false}')
    await ensureRepoDefaults(root)
    expect((await readRepoSettings(root)).autoRun).toBe(false)
  })

  it('preserves other keys when adding the default', async () => {
    await fs.mkdir(join(root, '.indiana', 'casablanca'), { recursive: true })
    await fs.writeFile(join(root, '.indiana', 'casablanca', 'settings.json'), '{"color":"1 2 3"}')
    await ensureRepoDefaults(root)
    const settings = await readRepoSettings(root)
    expect(settings.color).toBe('1 2 3')
    expect(settings.autoRun).toBe(true)
  })
})
