import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import { existsSync } from 'node:fs'
import { createConnection } from 'node:net'
import { homedir } from 'node:os'
import { join } from 'node:path'
import type {
  AgentJobsResult,
  AnswerAgentJobResult,
  CopyAllResult,
  ElicitationAction,
  VaultConfig
} from '@shared/domain'

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

const DAEMON_REQUEST_TIMEOUT_MS = 2_000

function daemonSocketPath(): string {
  return join(process.env.INDIANA_HOME || join(homedir(), '.indiana'), 'indiana.sock')
}

/** One request/response round trip to the local Indiana daemon. */
function daemonRequest<T>(request: object): Promise<T> {
  return new Promise((resolve, reject) => {
    const socket = createConnection(daemonSocketPath())
    let received = ''
    const timer = setTimeout(() => {
      socket.destroy()
      reject(new Error('Indiana daemon did not respond'))
    }, DAEMON_REQUEST_TIMEOUT_MS)
    const close = (): void => clearTimeout(timer)

    socket.once('connect', () => socket.write(`${JSON.stringify(request)}\n`))
    socket.on('data', (chunk: Buffer) => {
      received += chunk.toString()
      const newline = received.indexOf('\n')
      if (newline < 0) return
      close()
      socket.end()
      try {
        resolve(JSON.parse(received.slice(0, newline)) as T)
      } catch (err) {
        reject(err)
      }
    })
    socket.once('error', (err) => {
      close()
      reject(err)
    })
  })
}

/** Fetch live agent turns. A missing daemon is an ordinary offline state. */
export async function agentJobs(): Promise<AgentJobsResult> {
  try {
    const response = await daemonRequest<{ jobs: AgentJobsResult['jobs'] }>({ cmd: 'jobs' })
    return { online: true, jobs: response.jobs }
  } catch {
    return { online: false, jobs: [] }
  }
}

/** Forward one human choice to the ACP turn that asked for it. */
export function answerAgentJob(
  jobId: string,
  action: ElicitationAction,
  answer?: string
): Promise<AnswerAgentJobResult> {
  return daemonRequest<AnswerAgentJobResult>({
    cmd: 'answerjob',
    job_id: jobId,
    action,
    answer: answer ?? null
  })
}

/**
 * Ask Indiana to monitor a repo: `indiana add <root>` registers it in
 * `~/.indiana/config.json`, scaffolds `.indiana/`, and live-adds it to a running
 * daemon (IN_DAEMON.md). Idempotent and best-effort — a missing binary or daemon
 * must not block opening a folder, so failures are logged, not thrown.
 */
export async function ensureMonitored(rootPath: string): Promise<void> {
  const bin = resolveIndianaBinary()
  if (!bin) {
    console.warn('[indiana] binary not found — folder will not be auto-monitored')
    return
  }
  try {
    await execFileAsync(bin, ['add', rootPath], { timeout: 15_000 })
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    console.warn('[indiana] add failed:', e.stderr?.trim() || e.message || String(err))
  }
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
