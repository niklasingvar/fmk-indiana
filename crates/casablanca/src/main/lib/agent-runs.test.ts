import { promises as fs } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { readRun } from './agent-runs'

// Listing goes through `indiana runs --json` (grammar lives in Rust, tested
// there); only the raw-record reader is main-process logic worth pinning.
describe('readRun', () => {
  let root: string

  beforeEach(async () => {
    root = await fs.mkdtemp(join(tmpdir(), 'casa-runs-'))
  })

  afterEach(async () => {
    await fs.rm(root, { recursive: true, force: true })
  })

  it('reads a record back verbatim', async () => {
    const dir = join(root, '.indiana', 'chief-of-staff', 'runs')
    await fs.mkdir(dir, { recursive: true })
    await fs.writeFile(join(dir, '2026-07-19-211423-su-nak.md'), '# Run su-nak — done\n')
    expect(await readRun({ rootPath: root }, '2026-07-19-211423-su-nak.md')).toBe(
      '# Run su-nak — done\n'
    )
  })

  it('rejects path traversal in record names', async () => {
    await expect(readRun({ rootPath: root }, '../secrets.md')).rejects.toThrow(
      'invalid run record name'
    )
    await expect(readRun({ rootPath: root }, 'a/b.md')).rejects.toThrow('invalid run record name')
  })
})
