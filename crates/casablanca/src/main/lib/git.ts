/**
 * Git working-tree status for tree-row tinting. Read-only: one
 * `git status --porcelain` per refresh; no repo → empty map, no tints.
 */

import { execFile } from 'node:child_process'
import { promisify } from 'node:util'
import type { GitFileStatus, GitStatusMap, VaultConfig } from '@shared/domain'

const execFileP = promisify(execFile)

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
    const { stdout } = await execFileP(
      'git',
      ['-C', vault.rootPath, 'status', '--porcelain=v1', '--untracked-files=all'],
      { timeout: 5000, maxBuffer: 4 * 1024 * 1024 }
    )
    return parsePorcelain(stdout)
  } catch {
    return {}
  }
}
