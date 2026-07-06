import { ipcMain, dialog, BrowserWindow } from 'electron'
import { promises as fs } from 'node:fs'
import { IPC } from '@shared/ipc'
import type { AnnotationRequest, VaultConfig } from '@shared/domain'
import { getVaultConfig, setVaultConfig } from './lib/config'
import { copyAllMarkers } from './lib/indiana'
import { appendAnnotation } from './lib/annotations'
import { readTree, readNote, writeNote, createNote, deleteNote, toRelative } from './lib/vault'

type Sender = Pick<BrowserWindow, 'webContents'>

export interface IpcRegistration {
  /** Vault at startup, for the initial watcher/tree push. */
  vault: VaultConfig | null
  /** Live accessor — reflects vault switches after VAULT_CHOOSE/SET. */
  getVault: () => VaultConfig | null
}

/**
 * Register all main-process IPC handlers. The contract lives in @shared/ipc
 * and the preload bridge mirrors it. Each handler validates that a vault is
 * selected before touching the filesystem.
 */
export async function registerIpc(sender: Sender): Promise<IpcRegistration> {
  let vault = await getVaultConfig()

  const requireVault = (): VaultConfig => {
    if (!vault) throw new Error('No vault selected')
    return vault
  }

  const refresh = async (): Promise<void> => {
    if (vault) sender.webContents.send(IPC.TREE_CHANGED, await readTree(vault))
  }

  const handle = (channel: string, fn: (...args: unknown[]) => unknown): void => {
    ipcMain.handle(channel, async (_e, ...args) => {
      try {
        return await fn(...args)
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err)
        console.error(`[ipc:${channel}]`, message)
        throw new Error(message)
      }
    })
  }

  // --- vault lifecycle --------------------------------------------------

  handle(IPC.VAULT_GET, async () =>
    vault ? { status: 'ready', rootPath: vault.rootPath } : { status: 'unset' }
  )

  handle(IPC.VAULT_CHOOSE, async () => {
    const result = await dialog.showOpenDialog({
      properties: ['openDirectory', 'createDirectory']
    })
    if (result.canceled || result.filePaths.length === 0) return null
    const rootPath = result.filePaths[0]
    await fs.mkdir(rootPath, { recursive: true })
    vault = { rootPath }
    await setVaultConfig(vault)
    await refresh()
    return { status: 'ready', rootPath } as const
  })

  handle(IPC.VAULT_SET, async (rootPath: unknown) => {
    vault = { rootPath: String(rootPath) }
    await setVaultConfig(vault)
    await refresh()
    return { status: 'ready', rootPath: vault.rootPath } as const
  })

  // --- tree + notes -----------------------------------------------------

  handle(IPC.TREE_READ, async () => readTree(requireVault()))

  handle(IPC.NOTE_READ, async (rel: unknown) =>
    readNote(requireVault(), String(rel))
  )

  handle(IPC.NOTE_WRITE, async (rel: unknown, content: unknown) => {
    const note = await writeNote(requireVault(), String(rel), String(content))
    await refresh()
    return note
  })

  handle(IPC.NOTE_CREATE, async (dirRel: unknown, name: unknown) => {
    const note = await createNote(requireVault(), String(dirRel), String(name))
    await refresh()
    return note
  })

  handle(IPC.NOTE_DELETE, async (rel: unknown) => {
    await deleteNote(requireVault(), String(rel))
    await refresh()
  })

  // --- annotations --------------------------------------------------------

  handle(IPC.ANNOTATION_APPEND, async (req: unknown) => {
    const result = await appendAnnotation(requireVault(), req as AnnotationRequest)
    await refresh()
    return result
  })

  // --- indiana ------------------------------------------------------------

  handle(IPC.INDIANA_COPY_ALL, async () => copyAllMarkers(requireVault()))

  // Utility: convert an absolute path to a vault-relative one.
  handle('vault:rel', async (abs: unknown) => toRelative(requireVault(), String(abs)))

  return { vault, getVault: () => vault }
}
