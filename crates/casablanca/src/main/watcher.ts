import chokidar, { type FSWatcher } from 'chokidar'
import { relative, sep } from 'node:path'
// Type-only: keeps this module loadable outside Electron (vitest).
import type { BrowserWindow } from 'electron'
import { IPC } from '@shared/ipc'
import { readTree } from './lib/vault'
import { gitStatus } from './lib/git'
import type { TreeNode, VaultConfig } from '@shared/domain'

/**
 * Watches the vault folder for external changes and pushes a fresh tree to
 * the renderer. Debounced so a burst of saves produces one refresh. HTML
 * changes additionally push a per-path preview:changed so an open preview
 * reloads when the agent edits the document.
 */
export function watchVault(vault: VaultConfig, getWindow: () => BrowserWindow | null): FSWatcher {
  let timer: NodeJS.Timeout | null = null
  const previewTimers = new Map<string, NodeJS.Timeout>()

  const push = (): void => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => {
      void readTree(vault).then((tree: TreeNode) => {
        getWindow()?.webContents.send(IPC.TREE_CHANGED, tree)
      })
      void gitStatus(vault).then((map) => {
        getWindow()?.webContents.send(IPC.GIT_CHANGED, map)
      })
    }, 150)
  }

  const pushPreview = (rel: string): void => {
    const posix = rel.split(sep).join('/')
    const pending = previewTimers.get(posix)
    if (pending) clearTimeout(pending)
    previewTimers.set(
      posix,
      setTimeout(() => {
        previewTimers.delete(posix)
        getWindow()?.webContents.send(IPC.PREVIEW_CHANGED, posix)
      }, 150)
    )
  }

  // Markdown counterpart of pushPreview: lets an open note adopt external
  // edits — the Indiana daemon's marker claims and the agent's fixes — so the
  // buffer never goes stale and autosave never resurrects a resolved marker.
  const noteTimers = new Map<string, NodeJS.Timeout>()
  const pushNote = (rel: string): void => {
    const posix = rel.split(sep).join('/')
    const pending = noteTimers.get(posix)
    if (pending) clearTimeout(pending)
    noteTimers.set(
      posix,
      setTimeout(() => {
        noteTimers.delete(posix)
        getWindow()?.webContents.send(IPC.NOTE_CHANGED, posix)
      }, 150)
    )
  }

  const isRelevantFile = (posix: string): boolean =>
    /\.(md|mdx|html?)$/i.test(posix) || posix === '.indiana/casablanca/settings.json'

  // Chokidar v4 dropped glob support, so we watch the vault root and filter by
  // extension in `ignored` (dirs must pass through so recursion continues).
  return chokidar
    .watch('.', {
      cwd: vault.rootPath,
      ignoreInitial: true,
      // Dotfolders like .indiana are watched; only heavy/derived dirs are not.
      // `p` is absolute; `stats` is present for files once chokidar stats them.
      ignored: (p: string, stats) => {
        if (/(^|[/\\])(\.git|node_modules|target|dist|out)([/\\]|$)/.test(p)) return true
        if (!stats?.isFile()) return false
        return !isRelevantFile(relative(vault.rootPath, p).split(sep).join('/'))
      },
      awaitWriteFinish: { stabilityThreshold: 200, pollInterval: 50 }
    })
    .on('all', (event, path) => {
      const posix = path.split(sep).join('/')
      const isDirEvent = event === 'addDir' || event === 'unlinkDir'
      if (!isDirEvent && !isRelevantFile(posix)) return
      push()
      if (event === 'add' || event === 'change') {
        if (/\.html?$/i.test(posix)) pushPreview(path)
        if (/\.mdx?$/i.test(posix)) pushNote(path)
        // Theme (and other editor prefs) live in settings.json — push so the
        // renderer can re-read vault state without a project switch.
        if (posix === '.indiana/casablanca/settings.json') {
          pushNote(path)
        }
      }
    })
}
