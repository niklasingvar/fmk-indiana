/**
 * The vault:// protocol serves files from the current vault root so HTML
 * documents render in the preview iframe with their relative assets intact.
 * HTML responses get the annotator script injected — served same-origin at a
 * reserved path so pages restricted to `script-src 'self'` still annotate.
 */

import { protocol } from 'electron'
import { promises as fs } from 'node:fs'
import type { VaultConfig } from '@shared/domain'
import { mimeFor, resolveVaultPath } from './resolve-path'
import annotatorSource from './annotator.js?raw'

export const VAULT_SCHEME = 'vault'
/** Origin host — vault URLs look like vault://local/site/page.html. */
export const VAULT_HOST = 'local'
const ANNOTATOR_PATH = '/__casablanca__/annotator.js'
const ANNOTATOR_TAG = `<script src="${ANNOTATOR_PATH}"></script>`

/** Must run before app.whenReady() for relative URLs to resolve in-scheme. */
export function registerVaultSchemeAsPrivileged(): void {
  protocol.registerSchemesAsPrivileged([
    {
      scheme: VAULT_SCHEME,
      privileges: { standard: true, secure: true, supportFetchAPI: true, stream: true }
    }
  ])
}

export function injectAnnotator(html: string): string {
  const idx = html.search(/<\/body\s*>/i)
  if (idx === -1) return html + ANNOTATOR_TAG
  return html.slice(0, idx) + ANNOTATOR_TAG + html.slice(idx)
}

export function registerVaultProtocol(getVault: () => VaultConfig | null): void {
  protocol.handle(VAULT_SCHEME, async (request) => {
    const noStore = { 'Cache-Control': 'no-store' }
    const { pathname } = new URL(request.url)

    if (pathname === ANNOTATOR_PATH) {
      return new Response(annotatorSource, {
        headers: { ...noStore, 'Content-Type': 'text/javascript; charset=utf-8' }
      })
    }

    const vault = getVault()
    if (!vault) return new Response('no vault selected', { status: 503, headers: noStore })

    const abs = resolveVaultPath(vault.rootPath, pathname)
    if (!abs) return new Response('forbidden', { status: 403, headers: noStore })

    const mime = mimeFor(abs)
    try {
      if (mime.startsWith('text/html')) {
        const html = await fs.readFile(abs, 'utf8')
        return new Response(injectAnnotator(html), {
          headers: { ...noStore, 'Content-Type': mime }
        })
      }
      const body = await fs.readFile(abs)
      return new Response(new Uint8Array(body), {
        headers: { ...noStore, 'Content-Type': mime }
      })
    } catch {
      return new Response('not found', { status: 404, headers: noStore })
    }
  })
}
