import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import { existsSync } from 'node:fs'
import { homedir } from 'node:os'
import { join } from 'node:path'
import type { CopyAllResult, VaultConfig } from '@shared/domain'

const execFileAsync = promisify(execFile)

/**
 * Where the `indiana` binary lives. A GUI app's PATH is launchd's default,
 * not the user's shell PATH (docs/DISTRO.md — same lesson as the menulet),
 * so we check the standard install locations explicitly.
 */
const BINARY_CANDIDATES = [
  join(homedir(), '.local', 'bin', 'indiana'),
  '/opt/homebrew/bin/indiana',
  '/usr/local/bin/indiana'
]

export function resolveIndianaBinary(): string | null {
  return BINARY_CANDIDATES.find((p) => existsSync(p)) ?? null
}

/**
 * Run `indiana copy <vault root>`: compile every pending marker in the vault
 * and put the bundle on the clipboard. Indiana owns compilation and the
 * clipboard; Casablanca only triggers and reports.
 */
export async function copyAllMarkers(vault: VaultConfig): Promise<CopyAllResult> {
  const bin = resolveIndianaBinary()
  if (!bin) {
    return {
      ok: false,
      message: 'indiana not found — brew install niklasingvar/fmk-indiana/indiana'
    }
  }
  try {
    const { stdout, stderr } = await execFileAsync(bin, ['copy', vault.rootPath], {
      timeout: 15_000
    })
    const message = (stdout.trim() || stderr.trim()) || 'Copied to clipboard'
    return { ok: true, message }
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    const message = e.stderr?.trim() || e.message || String(err)
    return { ok: false, message }
  }
}
