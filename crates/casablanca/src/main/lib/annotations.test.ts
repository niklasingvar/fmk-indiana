import { promises as fs } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { appendAnnotation } from './annotations'
import type { AnnotationRequest, VaultConfig } from '@shared/domain'

let vault: VaultConfig

const req = (over: Partial<AnnotationRequest> = {}): AnnotationRequest => ({
  docRelPath: 'site/page.html',
  selector: 'main > h2',
  excerpt: 'Pricing tiers',
  kind: 'fix',
  message: 'align the columns',
  ...over
})

beforeEach(async () => {
  const root = await fs.mkdtemp(join(tmpdir(), 'casa-annotations-'))
  vault = { rootPath: root }
  await fs.mkdir(join(root, 'site'))
  await fs.writeFile(join(root, 'site/page.html'), '<html></html>')
})

afterEach(async () => {
  await fs.rm(vault.rootPath, { recursive: true, force: true })
})

describe('appendAnnotation', () => {
  it('creates the sidecar with a header on first annotation', async () => {
    const result = await appendAnnotation(vault, req())
    expect(result.sidecarRelPath).toBe('site/page.html.md')
    const content = await fs.readFile(join(vault.rootPath, 'site/page.html.md'), 'utf8')
    expect(content).toBe(
      '# Annotations — site/page.html\n\n' +
        '- [site/page.html] main > h2 — "Pricing tiers" ::fix align the columns\n'
    )
  })

  it('appends subsequent annotations as one line each', async () => {
    await appendAnnotation(vault, req())
    await appendAnnotation(vault, req({ kind: 'hate', selector: '#hero', excerpt: 'Casa' }))
    const content = await fs.readFile(join(vault.rootPath, 'site/page.html.md'), 'utf8')
    const lines = content.trimEnd().split('\n')
    expect(lines).toHaveLength(4)
    expect(lines[3]).toBe('- [site/page.html] #hero — "Casa" ::hate')
  })

  it('repairs a missing trailing newline before appending', async () => {
    await fs.writeFile(join(vault.rootPath, 'site/page.html.md'), '# hand-edited, no newline')
    await appendAnnotation(vault, req())
    const content = await fs.readFile(join(vault.rootPath, 'site/page.html.md'), 'utf8')
    expect(content).toBe(
      '# hand-edited, no newline\n' +
        '- [site/page.html] main > h2 — "Pricing tiers" ::fix align the columns\n'
    )
  })

  it('rejects non-html targets and vault escapes', async () => {
    await expect(appendAnnotation(vault, req({ docRelPath: 'notes.md' }))).rejects.toThrow(
      /not an html/
    )
    await expect(
      appendAnnotation(vault, req({ docRelPath: '../outside/page.html' }))
    ).rejects.toThrow(/escapes vault/)
  })

  it('propagates the required-message contract', async () => {
    await expect(appendAnnotation(vault, req({ kind: 'todo', message: '' }))).rejects.toThrow(
      /requires a message/
    )
  })
})
