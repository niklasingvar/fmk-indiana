import { promises as fs } from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { readTree } from './vault'

describe('readTree', () => {
  let root: string

  beforeEach(async () => {
    root = await fs.mkdtemp(join(tmpdir(), 'casa-vault-'))
  })

  afterEach(async () => {
    await fs.rm(root, { recursive: true, force: true })
  })

  it('includes JSON settings files', async () => {
    await fs.mkdir(join(root, '.indiana', 'casablanca'), { recursive: true })
    await fs.writeFile(join(root, '.indiana', 'casablanca', 'settings.json'), '{"autoRun":true}\n')

    expect(await readTree({ rootPath: root })).toEqual({
      path: '',
      name: basename(root),
      type: 'folder',
      children: [
        {
          path: '.indiana',
          name: '.indiana',
          type: 'folder',
          children: [
            {
              path: '.indiana/casablanca',
              name: 'casablanca',
              type: 'folder',
              children: [{ path: '.indiana/casablanca/settings.json', name: 'settings.json', type: 'file' }]
            }
          ]
        }
      ]
    })
  })
})
