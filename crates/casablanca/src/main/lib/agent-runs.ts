import { promises as fsp } from 'node:fs'
import { join } from 'node:path'
import type { AgentRun, AgentRunsResult, VaultConfig } from '@shared/domain'

/**
 * Read the durable agent-run audit records the Indiana daemon leaves under
 * `.indiana/chief-of-staff/runs/` (COS_PRD.md) — one markdown file per turn.
 * Straight from disk, like the agents roster: the records are a fact of the
 * folder, not of the daemon, so history stays browsable while it is offline.
 */

const RUNS_REL = '.indiana/chief-of-staff/runs'

/** Newest runs first; the panel is a recent-history view, not an archive. */
const MAX_RUNS = 100

/** Header line: `# Run <job-id> — <outcome>`. */
const TITLE = /^# Run (\S+) — (\w+)/m

/**
 * Parse one record's summary fields out of its markdown. Only the header
 * bullet list is read; the transcript below is fetched on selection.
 */
export function parseRunRecord(file: string, text: string): AgentRun {
  const title = TITLE.exec(text)
  const bullet = (name: string): string | undefined => {
    const match = new RegExp(`^- ${name}: (.+)$`, 'm').exec(text)
    return match?.[1].trim()
  }
  const outcomeLine = bullet('outcome')
  // `- outcome: failed (dispatch error: …)` → detail inside the parens.
  const detail = outcomeLine?.match(/\((.+)\)$/)?.[1]
  const outcome = title?.[2] === 'done' ? 'done' : title?.[2] === 'failed' ? 'failed' : 'unknown'
  return {
    file,
    jobId: title?.[1] ?? file.replace(/\.md$/, ''),
    outcome,
    started: bullet('started')?.replace(/ UTC$/, '') ?? '',
    detail,
    tokens: bullet('tokens'),
    cost: bullet('cost')
  }
}

/** List run records, newest first (timestamped filenames sort by time). */
export async function listRuns(vault: VaultConfig): Promise<AgentRunsResult> {
  const dir = join(vault.rootPath, RUNS_REL)
  let names: string[]
  try {
    names = (await fsp.readdir(dir)).filter((name) => name.endsWith('.md'))
  } catch {
    return { available: false, runs: [] }
  }
  names.sort().reverse()
  const runs = await Promise.all(
    names.slice(0, MAX_RUNS).map(async (name) => {
      const text = await fsp.readFile(join(dir, name), 'utf8').catch(() => '')
      return parseRunRecord(name, text)
    })
  )
  return { available: true, runs }
}

/** Full markdown of one record. `file` must be a bare filename from listRuns. */
export async function readRun(vault: VaultConfig, file: string): Promise<string> {
  if (file.includes('/') || file.includes('\\') || file.includes('..')) {
    throw new Error('invalid run record name')
  }
  return fsp.readFile(join(vault.rootPath, RUNS_REL, file), 'utf8')
}
