import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import { existsSync, promises as fsp } from 'node:fs'
import { createConnection } from 'node:net'
import { homedir } from 'node:os'
import { join } from 'node:path'
import type {
  AgentJobsResult,
  AnswerAgentJobResult,
  CopyAllResult,
  CosLogEntry,
  CosLogResult,
  CosTask,
  CosTasksResult,
  DispatchResult,
  ElicitationAction,
  JobTranscriptResult,
  MarkerStatus,
  TranscriptEvent,
  VaultAgentsResult,
  VaultConfig,
  VaultMarker,
  VaultMarkersResult
} from '@shared/domain'

const execFileAsync = promisify(execFile)

/**
 * Where the `indiana` binary lives. A GUI app's PATH is launchd's default,
 * not the user's shell PATH (docs/DISTRO.md — same lesson as the menulet),
 * so we check the standard install locations explicitly. The dev launcher
 * provides its repo binary through INDIANA_BIN.
 */
const STANDARD_BINARY_CANDIDATES = [
  join(homedir(), '.local', 'bin', 'indiana'),
  '/opt/homebrew/bin/indiana',
  '/usr/local/bin/indiana'
]

export function resolveIndianaBinary(): string | null {
  const candidates = process.env.INDIANA_BIN
    ? [process.env.INDIANA_BIN, ...STANDARD_BINARY_CANDIDATES]
    : STANDARD_BINARY_CANDIDATES
  return candidates.find((p) => existsSync(p)) ?? null
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

/**
 * Fetch a live turn's transcript from `sinceSeq` on. A gone job or offline
 * daemon both come back as `found: false` — the follow view treats either as
 * "turn ended".
 */
export async function jobTranscript(jobId: string, sinceSeq: number): Promise<JobTranscriptResult> {
  try {
    const response = await daemonRequest<{
      found: boolean
      events: TranscriptEvent[]
      next_seq: number
    }>({ cmd: 'jobtranscript', job_id: jobId, since_seq: sinceSeq })
    return { found: response.found, events: response.events, nextSeq: response.next_seq }
  } catch {
    return { found: false, events: [], nextSeq: sinceSeq }
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
 * Read the Chief of Staff tracker through `indiana task list --json` — the
 * line grammar exists in exactly one language (core computes, faces render).
 * A missing binary or a failed run degrades to an unavailable panel, never an
 * error dialog.
 */
export async function listTasks(vault: VaultConfig): Promise<CosTasksResult> {
  const bin = resolveIndianaBinary()
  if (!bin) return { available: false, tasks: [] }
  try {
    const { stdout } = await execFileAsync(
      bin,
      ['task', 'list', '--root', vault.rootPath, '--state', 'all', '--json'],
      { timeout: 10_000 }
    )
    return { available: true, tasks: JSON.parse(stdout) as CosTask[] }
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    console.warn('[indiana] task list failed:', e.stderr?.trim() || e.message || String(err))
    return { available: false, tasks: [] }
  }
}

/** Tail the Chief of Staff action log through `indiana log -n <n> --json`. */
export async function tailLog(vault: VaultConfig, lines: number): Promise<CosLogResult> {
  const bin = resolveIndianaBinary()
  if (!bin) return { available: false, entries: [] }
  try {
    const { stdout } = await execFileAsync(
      bin,
      ['log', '--root', vault.rootPath, '-n', String(lines), '--json'],
      { timeout: 10_000 }
    )
    return { available: true, entries: JSON.parse(stdout) as CosLogEntry[] }
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    console.warn('[indiana] log failed:', e.stderr?.trim() || e.message || String(err))
    return { available: false, entries: [] }
  }
}

/** The scan JSON's marker shape (indiana_core::index::Located). */
interface ScannedMarker {
  path: string
  line: number
  kind: string
  raw_token: string
  message?: string
  group?: number
  agent?: string
  id?: string
  status?: MarkerStatus
}

/** Project scan output onto the renderer's shape, paths made vault-relative. */
export function toVaultMarkers(scanJson: string, rootPath: string): VaultMarker[] {
  const root = rootPath.endsWith('/') ? rootPath : `${rootPath}/`
  const parsed = JSON.parse(scanJson) as { markers: ScannedMarker[] }
  return parsed.markers.map((m) => ({
    path: m.path.startsWith(root) ? m.path.slice(root.length) : m.path,
    line: m.line,
    kind: m.kind,
    rawToken: m.raw_token,
    message: m.message,
    group: m.group,
    agent: m.agent,
    id: m.id,
    status: m.status
  }))
}

/**
 * List every marker in the vault via `indiana scan --json --read-only` —
 * read-only so opening the overview never injects IDs or touches files.
 * A missing binary or failed scan degrades to an unavailable panel.
 */
export async function listMarkers(vault: VaultConfig): Promise<VaultMarkersResult> {
  const bin = resolveIndianaBinary()
  if (!bin) return { available: false, markers: [] }
  try {
    const { stdout } = await execFileAsync(
      bin,
      ['scan', '--json', '--read-only', vault.rootPath],
      { timeout: 15_000, maxBuffer: 16 * 1024 * 1024 }
    )
    return { available: true, markers: toVaultMarkers(stdout, vault.rootPath) }
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    console.warn('[indiana] scan failed:', e.stderr?.trim() || e.message || String(err))
    return { available: false, markers: [] }
  }
}

/**
 * Run `indiana copy <vault root> [filters]`: compile the selected markers and
 * put the bundle on the clipboard. Indiana owns compilation and the clipboard;
 * Casablanca only triggers and reports.
 */
async function copyMarkers(vault: VaultConfig, filters: string[]): Promise<CopyAllResult> {
  const bin = resolveIndianaBinary()
  if (!bin) {
    return {
      ok: false,
      message: 'indiana not found — brew install niklasingvar/fmk-indiana/indiana'
    }
  }
  try {
    const { stderr } = await execFileAsync(bin, ['copy', vault.rootPath, ...filters], {
      timeout: 15_000,
      maxBuffer: 16 * 1024 * 1024
    })
    return { ok: true, message: stderr.trim() || 'Copied to clipboard' }
  } catch (err) {
    const e = err as { stderr?: string; message?: string }
    const message = e.stderr?.trim() || e.message || String(err)
    return { ok: false, message }
  }
}

export function copyAllMarkers(vault: VaultConfig): Promise<CopyAllResult> {
  return copyMarkers(vault, [])
}

/** Copy one numeric batch (`::fix -1 …`) with the default system prompt. */
export function copyGroupMarkers(vault: VaultConfig, group: number): Promise<CopyAllResult> {
  return copyMarkers(vault, ['--group', String(group)])
}

/** Copy one agent's batch (`::fix -m …`) with that persona's system prompt. */
export function copyAgentMarkers(vault: VaultConfig, agent: string): Promise<CopyAllResult> {
  return copyMarkers(vault, ['--agent', agent])
}

/** Dispatch one numeric batch as a manual agent turn through the daemon. */
export async function runGroup(vault: VaultConfig, group: number): Promise<DispatchResult> {
  try {
    return await daemonRequest<DispatchResult>({
      cmd: 'rungroup',
      path: vault.rootPath,
      group
    })
  } catch {
    return { accepted: false, count: 0 }
  }
}

/** Dispatch one agent persona's batch as a manual agent turn through the daemon. */
export async function runAgent(vault: VaultConfig, agent: string): Promise<DispatchResult> {
  try {
    return await daemonRequest<DispatchResult>({
      cmd: 'runagent',
      path: vault.rootPath,
      agent
    })
  } catch {
    return { accepted: false, count: 0 }
  }
}

/** A directory name usable as an agent flag token (indiana_core::agents). */
const AGENT_NAME = /^[a-z][a-z0-9-]*$/

/**
 * The vault's agent personas: directories under `.indiana/agents/` carrying a
 * `SYSTEM_PROMPT.md`. Read straight from disk — the roster is a fact of the
 * folder, not of the daemon.
 */
export async function listAgents(vault: VaultConfig): Promise<VaultAgentsResult> {
  const dir = join(vault.rootPath, '.indiana', 'agents')
  try {
    const entries = await fsp.readdir(dir, { withFileTypes: true })
    const agents = entries
      .filter((entry) => entry.isDirectory() && AGENT_NAME.test(entry.name))
      .filter((entry) => existsSync(join(dir, entry.name, 'SYSTEM_PROMPT.md')))
      .map((entry) => entry.name)
      .sort()
    return { agents }
  } catch {
    return { agents: [] }
  }
}
