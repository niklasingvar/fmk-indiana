/**
 * Pure path resolution for the vault:// protocol — kept Electron-free so the
 * traversal guard is unit-testable. A URL pathname resolves to an absolute
 * file path strictly inside the vault root, or to null.
 */

import { resolve, sep } from 'node:path'

const MIME: Record<string, string> = {
  html: 'text/html; charset=utf-8',
  htm: 'text/html; charset=utf-8',
  css: 'text/css; charset=utf-8',
  js: 'text/javascript; charset=utf-8',
  mjs: 'text/javascript; charset=utf-8',
  json: 'application/json',
  svg: 'image/svg+xml',
  png: 'image/png',
  jpg: 'image/jpeg',
  jpeg: 'image/jpeg',
  gif: 'image/gif',
  webp: 'image/webp',
  ico: 'image/x-icon',
  woff: 'font/woff',
  woff2: 'font/woff2',
  ttf: 'font/ttf',
  txt: 'text/plain; charset=utf-8',
  md: 'text/plain; charset=utf-8'
}

export function mimeFor(path: string): string {
  const ext = path.slice(path.lastIndexOf('.') + 1).toLowerCase()
  return MIME[ext] ?? 'application/octet-stream'
}

export function resolveVaultPath(rootAbs: string, urlPathname: string): string | null {
  let decoded: string
  try {
    decoded = decodeURIComponent(urlPathname)
  } catch {
    return null
  }
  if (decoded.includes('\0') || decoded.includes('\\')) return null
  const rel = decoded.replace(/^\/+/, '')
  if (rel === '') return null
  if (rel.split('/').some((seg) => seg === '..')) return null
  const root = resolve(rootAbs)
  const abs = resolve(root, rel)
  if (!abs.startsWith(root + sep)) return null
  return abs
}
