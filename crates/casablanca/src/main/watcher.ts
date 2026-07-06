import chokidar, { type FSWatcher } from 'chokidar'
import { sep } from 'node:path'
import { BrowserWindow } from 'electron'
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

  return chokidar
    .watch(['**/*.md', '**/*.mdx', '**/*.html', '**/*.htm'], {
      cwd: vault.rootPath,
      ignoreInitial: true,
      // Dotfolders like .indiana are watched; only heavy/derived dirs are not.
      ignored: (p: string) => /(^|[/\\])(\.git|node_modules|target|dist|out)([/\\]|$)/.test(p),
      awaitWriteFinish: { stabilityThreshold: 200, pollInterval: 50 }
    })
    .on('all', (event, path) => {
      push()
      if ((event === 'add' || event === 'change') && /\.html?$/i.test(path)) pushPreview(path)
    })
}
