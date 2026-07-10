import { promises as fs } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import type { VaultConfig } from '@shared/domain'
import { deleteEntry } from './file-operations'

let vault: VaultConfig

beforeEach(async () => {
  vault = { rootPath: await fs.mkdtemp(join(tmpdir(), 'casa-files-')) }
})

afterEach(async () => {
  await fs.rm(vault.rootPath, { recursive: true, force: true })
})

describe('deleteEntry', () => {
  it('sends a file to the trash boundary', async () => {
    const file = join(vault.rootPath, 'notes.md')
    await fs.writeFile(file, '# Notes\n')
    let trashedPath: string | null = null

    await deleteEntry(vault, 'notes.md', async (path) => {
      trashedPath = path
    })

    expect(trashedPath).toBe(file)
    await expect(fs.stat(file)).resolves.toBeTruthy()
  })

  it('sends a whole folder to the trash boundary', async () => {
    const folder = join(vault.rootPath, 'docs')
    await fs.mkdir(join(folder, 'nested'), { recursive: true })
    await fs.writeFile(join(folder, 'nested/note.md'), '# Note\n')
    let trashedPath: string | null = null

    await deleteEntry(vault, 'docs', async (path) => {
      trashedPath = path
    })

    expect(trashedPath).toBe(folder)
  })

  it('rejects the root and paths outside the vault', async () => {
    const trash = async (): Promise<void> => undefined

    await expect(deleteEntry(vault, '', trash)).rejects.toThrow(/Invalid vault entry path/)
    await expect(deleteEntry(vault, '../outside', trash)).rejects.toThrow(/escapes vault/)
    await expect(deleteEntry(vault, vault.rootPath, trash)).rejects.toThrow(/escapes vault/)
  })
})
