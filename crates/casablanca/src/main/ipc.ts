import { ipcMain, dialog, BrowserWindow, shell } from 'electron'
import { promises as fs } from 'node:fs'
import { IPC } from '@shared/ipc'
import type { AnnotationRequest, ElicitationAction, VaultConfig, VaultState } from '@shared/domain'
import {
  getActiveProject,
  listProjects,
  addProject,
  setActiveProject,
  setProjectColor,
  removeProject
} from './lib/config'
import type { ProjectRecord } from '@shared/projects'
import { agentJobs, answerAgentJob, copyAllMarkers, ensureMonitored } from './lib/indiana'
import { ensureRepoDefaults } from './lib/repo-settings'
import { appendAnnotation } from './lib/annotations'
import { ensureRepo, gitDiffCommit, gitDiffHead, gitLog, gitStatus } from './lib/git'
import { deleteEntry, revealEntry } from './lib/file-operations'
import { readTree, readNote, writeNote, createNote, toRelative } from './lib/vault'

type Sender = Pick<BrowserWindow, 'webContents'>

export interface IpcRegistration {
  /** Vault at startup, for the initial watcher/tree push. */
  vault: VaultConfig | null
  /** Live accessor — reflects project switches. Read by the vault:// protocol. */
  getVault: () => VaultConfig | null
}

export interface IpcDeps {
  /** Point the file watcher at a new root (or tear it down when null). */
  retargetWatcher: (vault: VaultConfig | null) => void
}

/**
 * Register all main-process IPC handlers. The contract lives in @shared/ipc
 * and the preload bridge mirrors it. Each handler validates that a project is
 * active before touching the filesystem.
 */
export async function registerIpc(sender: Sender, deps: IpcDeps): Promise<IpcRegistration> {
  let active = await getActiveProject()
  let vault: VaultConfig | null = active ? { rootPath: active.rootPath } : null
  if (vault) await ensureRepo(vault)

  const requireVault = (): VaultConfig => {
    if (!vault) throw new Error('No project selected')
    return vault
  }

  const readyState = (a: ProjectRecord): VaultState => ({
    status: 'ready',
    rootPath: a.rootPath,
    color: a.color
  })

  /** Adopt a new active project: update state, re-target the watcher, push a refresh. */
  const adopt = async (a: ProjectRecord | null): Promise<VaultState> => {
    active = a
    vault = a ? { rootPath: a.rootPath } : null
    if (vault) {
      // Every open provisions what the repo needs, idempotently:
      // 1. git-backed: init + snapshot when no repo exists yet.
      await ensureRepo(vault)
      // 2. Indiana monitors it: registers in the daemon config, scaffolds
      //    .indiana/, starts watching.
      await ensureMonitored(vault.rootPath)
      // 3. Per-repo defaults (e.g. autoRun on) for any key not already set —
      //    never clobbers a deliberate choice.
      await ensureRepoDefaults(vault.rootPath)
    }
    deps.retargetWatcher(vault)
    await refresh()
    return a ? readyState(a) : { status: 'unset' }
  }

  const refresh = async (): Promise<void> => {
    if (!vault) return
    sender.webContents.send(IPC.TREE_CHANGED, await readTree(vault))
    sender.webContents.send(IPC.GIT_CHANGED, await gitStatus(vault))
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

  // --- project lifecycle ------------------------------------------------

  handle(IPC.VAULT_GET, async () => (active ? readyState(active) : { status: 'unset' }))

  handle(IPC.PROJECTS_LIST, async () => listProjects())

  // Pick a folder, register it, and switch to it. Null when the user cancels.
  handle(IPC.PROJECTS_ADD, async () => {
    const result = await dialog.showOpenDialog({
      properties: ['openDirectory', 'createDirectory']
    })
    if (result.canceled || result.filePaths.length === 0) return null
    const rootPath = result.filePaths[0]
    await fs.mkdir(rootPath, { recursive: true })
    // adopt() provisions the repo (monitor + .indiana scaffold + default autoRun on).
    return adopt(await addProject(rootPath))
  })

  handle(IPC.PROJECTS_SWITCH, async (rootPath: unknown) =>
    adopt(await setActiveProject(String(rootPath)))
  )

  // Recolor a project; return the fresh list so the renderer can repaint.
  handle(IPC.PROJECTS_SET_COLOR, async (rootPath: unknown, color: unknown) => {
    await setProjectColor(String(rootPath), String(color))
    if (active && active.rootPath === String(rootPath)) active = { ...active, color: String(color) }
    return listProjects()
  })

  handle(IPC.PROJECTS_REMOVE, async (rootPath: unknown) =>
    adopt(await removeProject(String(rootPath)))
  )

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

  handle(IPC.ENTRY_DELETE, async (rel: unknown) => {
    await deleteEntry(requireVault(), String(rel), (absolutePath) => shell.trashItem(absolutePath))
    await refresh()
  })

  handle(IPC.ENTRY_REVEAL, async (rel: unknown) => {
    await revealEntry(requireVault(), String(rel), (absolutePath) => shell.showItemInFolder(absolutePath))
  })

  // --- git history --------------------------------------------------------

  handle(IPC.GIT_LOG, async (rel: unknown) => gitLog(requireVault(), String(rel)))

  handle(IPC.GIT_DIFF_COMMIT, async (rel: unknown, hash: unknown) =>
    gitDiffCommit(requireVault(), String(rel), String(hash))
  )

  handle(IPC.GIT_DIFF_HEAD, async (rel: unknown) => gitDiffHead(requireVault(), String(rel)))

  // --- annotations --------------------------------------------------------

  handle(IPC.ANNOTATION_APPEND, async (req: unknown) => {
    const result = await appendAnnotation(requireVault(), req as AnnotationRequest)
    await refresh()
    return result
  })

  // --- indiana ------------------------------------------------------------

  handle(IPC.INDIANA_COPY_ALL, async () => copyAllMarkers(requireVault()))
  handle(IPC.INDIANA_JOBS, async () => agentJobs())
  handle(IPC.INDIANA_ANSWER_JOB, async (jobId: unknown, action: unknown, answer: unknown) =>
    answerAgentJob(
      String(jobId),
      action as ElicitationAction,
      typeof answer === 'string' ? answer : undefined
    )
  )

  // Utility: convert an absolute path to a vault-relative one.
  handle('vault:rel', async (abs: unknown) => toRelative(requireVault(), String(abs)))

  return { vault, getVault: () => vault }
}
