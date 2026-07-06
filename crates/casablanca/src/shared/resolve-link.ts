/**
 * Vault-internal link resolution for the editor: a relative href in a note
 * resolves against the note's folder to a vault-relative path the app can
 * open. External schemes go to the OS browser instead.
 */

export function isExternalLink(href: string): boolean {
  return /^[a-z][a-z0-9+.-]*:/i.test(href) || href.startsWith('//')
}

/**
 * Resolve `href` written in the note at `fromPath`. Returns the vault-relative
 * target, or null when the link is external, empty, or escapes the vault.
 * Extensionless targets get `.md` appended (wiki-style convenience).
 */
export function resolveVaultLink(fromPath: string, href: string): string | null {
  if (isExternalLink(href)) return null
  const clean = href.split(/[?#]/)[0]
  if (!clean) return null
  let decoded: string
  try {
    decoded = decodeURIComponent(clean)
  } catch {
    decoded = clean
  }

  // Leading slash = vault-root-relative; otherwise relative to the note's folder.
  const segs = decoded.startsWith('/') ? [] : fromPath.split('/').slice(0, -1)
  for (const part of decoded.replace(/^\/+/, '').split('/')) {
    if (part === '' || part === '.') continue
    if (part === '..') {
      if (segs.length === 0) return null
      segs.pop()
    } else {
      segs.push(part)
    }
  }
  const resolved = segs.join('/')
  if (!resolved) return null
  return /\.(mdx?|html?)$/i.test(resolved) ? resolved : `${resolved}.md`
}

/**
 * The inverse: the relative href to write in the note at `fromPath` so it
 * resolves to `toPath`. Same-or-deeper targets get a `./` prefix.
 */
export function relativeLink(fromPath: string, toPath: string): string {
  const fromDirs = fromPath.split('/').slice(0, -1)
  const toParts = toPath.split('/')
  let common = 0
  while (
    common < fromDirs.length &&
    common < toParts.length - 1 &&
    fromDirs[common] === toParts[common]
  ) {
    common++
  }
  const ups = fromDirs.length - common
  const rest = toParts.slice(common).join('/')
  return ups === 0 ? `./${rest}` : `${'../'.repeat(ups)}${rest}`
}
