/**
 * Git access for the vault. Read-only — status for tree-row tinting, log and
 * diffs for the per-note history panel — with one deliberate exception:
 * `ensureRepo` runs `git init` plus an initial snapshot commit so every
 * project has a baseline to diff against. Ongoing commits are the coding
 * agent's job (instructed by the Indiana loop preamble), never Casablanca's.
 */

import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import type { GitFileStatus, GitLogEntry, GitStatusMap, VaultConfig } from '@shared/domain'

const execFileP = promisify(execFile)

const EXEC_OPTS = { timeout: 5000, maxBuffer: 4 * 1024 * 1024 }

async function git(rootPath: string, args: string[]): Promise<string> {
  const { stdout } = await execFileP('git', ['-C', rootPath, ...args], EXEC_OPTS)
  return stdout
}

/** Folder aggregation: any modified child wins, then new, then deleted. */
const PRIORITY: Record<GitFileStatus, number> = { modified: 3, new: 2, deleted: 1 }

export function parsePorcelain(stdout: string): GitStatusMap {
  const map: GitStatusMap = {}
  for (const line of stdout.split('\n')) {
    if (line.length < 4) continue
    const x = line[0]
    const y = line[1]
    let path = line.slice(3)
    const arrow = path.indexOf(' -> ')
    if (arrow !== -1) path = path.slice(arrow + 4)
    if (path.startsWith('"') && path.endsWith('"')) path = path.slice(1, -1)
    const status: GitFileStatus =
      x === '?' || x === 'A' ? 'new' : x === 'D' || y === 'D' ? 'deleted' : 'modified'
    bump(map, path, status)
    const parts = path.split('/')
    for (let i = 1; i < parts.length; i++) bump(map, parts.slice(0, i).join('/'), status)
  }
  return map
}

function bump(map: GitStatusMap, path: string, status: GitFileStatus): void {
  const prev = map[path]
  if (!prev || PRIORITY[status] > PRIORITY[prev]) map[path] = status
}

export async function gitStatus(vault: VaultConfig): Promise<GitStatusMap> {
  try {
    return parsePorcelain(
      await git(vault.rootPath, ['status', '--porcelain=v1', '--untracked-files=all'])
    )
  } catch {
    return {}
  }
}

/**
 * One-time setup: if the project folder is not inside a git repo (own or
 * ancestor), initialize one and commit an initial snapshot so diffs and
 * history have a baseline. Idempotent; failures (e.g. no git identity
 * configured) are logged and non-fatal — the app degrades to no tints.
 */
export async function ensureRepo(vault: VaultConfig): Promise<void> {
  try {
    await git(vault.rootPath, ['rev-parse', '--is-inside-work-tree'])
    return
  } catch {
    // Not a repo — fall through to init.
  }
  try {
    await git(vault.rootPath, ['init'])
    await git(vault.rootPath, ['add', '-A'])
    await git(vault.rootPath, ['commit', '-m', 'casablanca: initial snapshot'])
  } catch (err) {
    console.error('[git] ensureRepo failed', err instanceof Error ? err.message : err)
  }
}

/** Parse `git log --format=%H%x09%ct%x09%s` output (tab-separated). */
export function parseLog(stdout: string): GitLogEntry[] {
  const entries: GitLogEntry[] = []
  for (const line of stdout.split('\n')) {
    const [hash, epoch, ...subject] = line.split('\t')
    if (!hash || !epoch) continue
    entries.push({ hash, timestamp: Number(epoch) * 1000, subject: subject.join('\t') })
  }
  return entries
}

/** Commits that touched the file, newest first, following renames. */
export async function gitLog(vault: VaultConfig, rel: string): Promise<GitLogEntry[]> {
  try {
    return parseLog(
      await git(vault.rootPath, ['log', '--follow', '--format=%H%x09%ct%x09%s', '--', rel])
    )
  } catch {
    return []
  }
}

/** Unified diff of what a single commit did to the file. */
export async function gitDiffCommit(
  vault: VaultConfig,
  rel: string,
  hash: string
): Promise<string> {
  try {
    return await git(vault.rootPath, ['show', hash, '--format=', '--patch', '--', rel])
  } catch {
    return ''
  }
}

/**
 * Unified diff of the file's uncommitted changes against HEAD. Untracked
 * files are invisible to `diff HEAD`, so those get an all-added diff
 * synthesized via `--no-index` (which exits 1 when the files differ).
 */
export async function gitDiffHead(vault: VaultConfig, rel: string): Promise<string> {
  try {
    const diff = await git(vault.rootPath, ['diff', 'HEAD', '--', rel])
    if (diff !== '') return diff
    const status = await git(vault.rootPath, ['status', '--porcelain=v1', '--', rel])
    if (!status.startsWith('??')) return ''
    return await git(vault.rootPath, ['diff', '--no-index', '--', '/dev/null', rel]).catch(
      (err: unknown) =>
        err && typeof err === 'object' && 'stdout' in err ? String(err.stdout) : ''
    )
  } catch {
    return ''
  }
}
