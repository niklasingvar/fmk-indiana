/**
 * Per-repo Casablanca settings at `<root>/.indiana/casablanca/settings.json`
 * (CASABLANCA_OVERVIEW.md). A committable JSON bag shared with the `indiana`
 * CLI (`indiana casablanca get/set`). The editor reads the keys it knows —
 * currently the project `color` — and ignores the rest. Missing/invalid file
 * degrades to no overrides.
 */

import { join } from 'node:path'
import { promises as fs } from 'node:fs'

export function repoSettingsPath(rootPath: string): string {
  return join(rootPath, '.indiana', 'casablanca', 'settings.json')
}

/** A JSON object, or `{}` for anything that isn't one (pure — for tests). */
export function parseRepoSettings(raw: string): Record<string, unknown> {
  try {
    const parsed: unknown = JSON.parse(raw)
    return parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : {}
  } catch {
    return {}
  }
}

/** The `color` override if the settings define a non-empty string (pure). */
export function repoColorOf(settings: Record<string, unknown>): string | null {
  const color = settings.color
  return typeof color === 'string' && color.trim() ? color : null
}

export async function readRepoSettings(rootPath: string): Promise<Record<string, unknown>> {
  try {
    return parseRepoSettings(await fs.readFile(repoSettingsPath(rootPath), 'utf8'))
  } catch {
    return {}
  }
}

export async function readRepoColor(rootPath: string): Promise<string | null> {
  return repoColorOf(await readRepoSettings(rootPath))
}

/** Merge one key into the settings file, creating the folder if needed. Kept
 * byte-compatible with the CLI's writer (2-space JSON + trailing newline). */
export async function writeRepoSetting(
  rootPath: string,
  key: string,
  value: unknown
): Promise<void> {
  const next = { ...(await readRepoSettings(rootPath)), [key]: value }
  await fs.mkdir(join(rootPath, '.indiana', 'casablanca'), { recursive: true })
  await fs.writeFile(repoSettingsPath(rootPath), JSON.stringify(next, null, 2) + '\n', 'utf8')
}
