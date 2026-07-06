/**
 * Appends annotation marker lines to the sidecar markdown next to an HTML
 * document. Plain fs only — Casablanca never parses markers; indiana scans
 * the sidecar like any other markdown in the vault.
 */

import { promises as fs } from 'node:fs'
import { resolve, sep } from 'node:path'
import type { AnnotationRequest, AnnotationResult, VaultConfig } from '@shared/domain'
import { buildAnnotationLine, isHtmlPath, sidecarHeader, sidecarPath } from '@shared/annotation-line'

export async function appendAnnotation(
  vault: VaultConfig,
  req: AnnotationRequest
): Promise<AnnotationResult> {
  if (!isHtmlPath(req.docRelPath)) throw new Error(`not an html document: ${req.docRelPath}`)
  const root = resolve(vault.rootPath)
  const docAbs = resolve(root, req.docRelPath)
  if (!docAbs.startsWith(root + sep)) throw new Error(`path escapes vault: ${req.docRelPath}`)

  const line = buildAnnotationLine(req)
  const sidecarRel = sidecarPath(req.docRelPath)
  const sidecarAbs = `${docAbs}.md`

  let existing: string | null
  try {
    existing = await fs.readFile(sidecarAbs, 'utf8')
  } catch {
    existing = null
  }

  const content =
    existing === null || existing === ''
      ? `${sidecarHeader(req.docRelPath)}\n${line}\n`
      : `${existing}${existing.endsWith('\n') ? '' : '\n'}${line}\n`

  await fs.writeFile(sidecarAbs, content, 'utf8')
  return { sidecarRelPath: sidecarRel }
}
