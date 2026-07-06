import { describe, expect, it } from 'vitest'
import { isExternalLink, relativeLink, resolveVaultLink } from './resolve-link'

describe('isExternalLink', () => {
  it('detects schemes and protocol-relative urls', () => {
    expect(isExternalLink('https://example.com')).toBe(true)
    expect(isExternalLink('mailto:a@b.c')).toBe(true)
    expect(isExternalLink('//cdn.example.com/x')).toBe(true)
    expect(isExternalLink('./other.md')).toBe(false)
    expect(isExternalLink('docs/spec.md')).toBe(false)
  })
})

describe('resolveVaultLink', () => {
  it('resolves relative to the note folder', () => {
    expect(resolveVaultLink('docs/guide.md', './other.md')).toBe('docs/other.md')
    expect(resolveVaultLink('docs/guide.md', 'sub/deep.md')).toBe('docs/sub/deep.md')
    expect(resolveVaultLink('docs/guide.md', '../top.md')).toBe('top.md')
  })

  it('treats a leading slash as vault-root-relative', () => {
    expect(resolveVaultLink('docs/deep/guide.md', '/readme.md')).toBe('readme.md')
  })

  it('appends .md to extensionless targets', () => {
    expect(resolveVaultLink('docs/guide.md', './other')).toBe('docs/other.md')
  })

  it('keeps html targets and decodes percent-encoding', () => {
    expect(resolveVaultLink('docs/guide.md', '../site/page.html')).toBe('site/page.html')
    expect(resolveVaultLink('docs/guide.md', 'my%20file.md')).toBe('docs/my file.md')
  })

  it('drops fragments and queries', () => {
    expect(resolveVaultLink('docs/guide.md', './other.md#section')).toBe('docs/other.md')
  })

  it('returns null for external, empty, and vault-escaping links', () => {
    expect(resolveVaultLink('docs/guide.md', 'https://x.y')).toBeNull()
    expect(resolveVaultLink('docs/guide.md', '#anchor')).toBeNull()
    expect(resolveVaultLink('top.md', '../../etc/passwd')).toBeNull()
  })
})

describe('relativeLink', () => {
  it('writes hrefs that resolveVaultLink round-trips', () => {
    const cases: Array<[string, string]> = [
      ['docs/a.md', 'docs/b.md'],
      ['a.md', 'docs/x.md'],
      ['docs/deep/a.md', 'top.md'],
      ['docs/a.md', 'docs/deep/b.md'],
      ['docs/deep/a.md', 'site/page.html']
    ]
    for (const [from, to] of cases) {
      expect(resolveVaultLink(from, relativeLink(from, to))).toBe(to)
    }
  })

  it('prefers ./ for siblings and ../ to climb', () => {
    expect(relativeLink('docs/a.md', 'docs/b.md')).toBe('./b.md')
    expect(relativeLink('docs/deep/a.md', 'top.md')).toBe('../../top.md')
  })
})
