import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import { promises as fsp } from 'node:fs'
import { join } from 'node:path'
import type { AgentRun, AgentRunsResult, VaultConfig } from '@shared/domain'
import { resolveIndianaBinary } from './indiana'

const execFileAsync = promisify(execFile)

/**
 * The agent-run audit records the Indiana daemon leaves under
 * `.indiana/chief-of-staff/runs/` (COS_PRD.md). The list goes through
 * `indiana runs --json` — the record grammar exists in exactly one language
 * (core computes, faces render), same stance as the tasks panel. Only the
 * selected record's raw markdown is read from disk, for display verbatim.
 */

const RUNS_REL = '.indiana/chief-of-staff/runs'

/** Newest runs first; the panel is a recent-history view, not an archive. */
const MAX_RUNS = 100

/** List run records, newest first. Missing binary or CLI failure degrades. */
export async function listRuns(vault: VaultConfig): Promise<AgentRunsResult> {
  const bin = resolveIndianaBinary()
  if (!bin) return { available: false, runs: [] }
  try {
    const { stdout } = await execFileAsync(
      bin,
      ['runs', '--root', vault.rootPath, '-n', String(MAX_RUNS), '--json'],
      { timeout: 10_000, maxBuffer: 16 * 1024 * 1024 }
    )
    return { available: true, runs: JSON.parse(stdout) as AgentRun[] }
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    console.warn('[indiana] runs failed:', e.stderr?.trim() || e.message || String(err))
    return { available: false, runs: [] }
  }
}

/** Full markdown of one record. `file` must be a bare filename from listRuns. */
export async function readRun(vault: VaultConfig, file: string): Promise<string> {
  if (file.includes('/') || file.includes('\\') || file.includes('..')) {
    throw new Error('invalid run record name')
  }
  return fsp.readFile(join(vault.rootPath, RUNS_REL, file), 'utf8')
}
