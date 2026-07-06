import { describe, expect, it } from 'vitest'
import { mimeFor, resolveVaultPath } from './resolve-path'

const ROOT = '/vault'

describe('resolveVaultPath', () => {
  it('resolves plain and nested paths inside the root', () => {
    expect(resolveVaultPath(ROOT, '/page.html')).toBe('/vault/page.html')
    expect(resolveVaultPath(ROOT, '/site/deep/style.css')).toBe('/vault/site/deep/style.css')
  })

  it('decodes percent-encoded segments', () => {
    expect(resolveVaultPath(ROOT, '/my%20site/page.html')).toBe('/vault/my site/page.html')
  })

  it('rejects traversal, plain and encoded', () => {
    expect(resolveVaultPath(ROOT, '/../etc/passwd')).toBeNull()
    expect(resolveVaultPath(ROOT, '/site/../../etc/passwd')).toBeNull()
    expect(resolveVaultPath(ROOT, '/%2e%2e/etc/passwd')).toBeNull()
    expect(resolveVaultPath(ROOT, '/site/%2e%2e/%2e%2e/etc/passwd')).toBeNull()
  })

  it('rejects backslashes, null bytes, empty, and bad encoding', () => {
    expect(resolveVaultPath(ROOT, '/a\\b.html')).toBeNull()
    expect(resolveVaultPath(ROOT, '/a%00.html')).toBeNull()
    expect(resolveVaultPath(ROOT, '/')).toBeNull()
    expect(resolveVaultPath(ROOT, '/%zz')).toBeNull()
  })

  it('treats leading slashes as root-relative, never absolute', () => {
    expect(resolveVaultPath(ROOT, '//etc/passwd')).toBe('/vault/etc/passwd')
  })
})

describe('mimeFor', () => {
  it('maps common extensions and defaults to octet-stream', () => {
    expect(mimeFor('/a/page.HTML')).toContain('text/html')
    expect(mimeFor('/a/style.css')).toContain('text/css')
    expect(mimeFor('/a/app.mjs')).toContain('text/javascript')
    expect(mimeFor('/a/logo.svg')).toBe('image/svg+xml')
    expect(mimeFor('/a/unknown.xyz')).toBe('application/octet-stream')
  })
})
