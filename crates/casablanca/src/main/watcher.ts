import chokidar, { type FSWatcher } from 'chokidar'
import { BrowserWindow } from 'electron'
import { IPC } from '@shared/ipc'
import { readTree } from './lib/vault'
import type { TreeNode, VaultConfig } from '@shared/domain'

/**
 * Watches the vault folder for external changes and pushes a fresh tree to
 * the renderer. Debounced so a burst of saves produces one refresh.
 */
export function watchVault(vault: VaultConfig, getWindow: () => BrowserWindow | null): FSWatcher {
  let timer: NodeJS.Timeout | null = null

  const push = (): void => {
    if (timer) clearTimeout(timer)
    timer = setTimeout(() => {
      void readTree(vault).then((tree: TreeNode) => {
        getWindow()?.webContents.send(IPC.TREE_CHANGED, tree)
      })
    }, 150)
  }

  return chokidar.watch(['**/*.md', '**/*.mdx'], {
    cwd: vault.rootPath,
    ignoreInitial: true,
    ignored: /(^|[/\\])\./,
    awaitWriteFinish: { stabilityThreshold: 200, pollInterval: 50 }
  }).on('all', push)
}
