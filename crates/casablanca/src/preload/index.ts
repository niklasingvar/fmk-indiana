import { contextBridge, ipcRenderer } from 'electron'
import { IPC } from '@shared/ipc'
import type {
  AnnotationRequest,
  AnnotationResult,
  CopyAllResult,
  GitLogEntry,
  GitStatusMap,
  Note,
  Project,
  TreeNode,
  VaultState
} from '@shared/domain'

/**
 * The renderer's only gateway to the system. Every capability the UI has is
 * declared here explicitly — nothing else is exposed. Keep this surface small.
 */
const api = {
  vault: {
    get: (): Promise<VaultState> => ipcRenderer.invoke(IPC.VAULT_GET)
  },
  projects: {
    list: (): Promise<Project[]> => ipcRenderer.invoke(IPC.PROJECTS_LIST),
    /** Opens a folder picker; null when cancelled. */
    add: (): Promise<VaultState | null> => ipcRenderer.invoke(IPC.PROJECTS_ADD),
    switch: (rootPath: string): Promise<VaultState> =>
      ipcRenderer.invoke(IPC.PROJECTS_SWITCH, rootPath),
    setColor: (rootPath: string, color: string): Promise<Project[]> =>
      ipcRenderer.invoke(IPC.PROJECTS_SET_COLOR, rootPath, color),
    remove: (rootPath: string): Promise<VaultState> =>
      ipcRenderer.invoke(IPC.PROJECTS_REMOVE, rootPath)
  },
  tree: {
    read: (): Promise<TreeNode> => ipcRenderer.invoke(IPC.TREE_READ),
    onChanged: (cb: (tree: TreeNode) => void): (() => void) => {
      const listener = (_e: unknown, tree: TreeNode): void => cb(tree)
      ipcRenderer.on(IPC.TREE_CHANGED, listener)
      return () => ipcRenderer.removeListener(IPC.TREE_CHANGED, listener)
    }
  },
  notes: {
    read: (rel: string): Promise<Note> => ipcRenderer.invoke(IPC.NOTE_READ, rel),
    write: (rel: string, content: string): Promise<Note> =>
      ipcRenderer.invoke(IPC.NOTE_WRITE, rel, content),
    create: (dirRel: string, name: string): Promise<Note> =>
      ipcRenderer.invoke(IPC.NOTE_CREATE, dirRel, name),
  },
  entries: {
    remove: (rel: string): Promise<void> => ipcRenderer.invoke(IPC.ENTRY_DELETE, rel)
  },
  annotations: {
    append: (req: AnnotationRequest): Promise<AnnotationResult> =>
      ipcRenderer.invoke(IPC.ANNOTATION_APPEND, req)
  },
  preview: {
    onChanged: (cb: (relPath: string) => void): (() => void) => {
      const listener = (_e: unknown, relPath: string): void => cb(relPath)
      ipcRenderer.on(IPC.PREVIEW_CHANGED, listener)
      return () => ipcRenderer.removeListener(IPC.PREVIEW_CHANGED, listener)
    }
  },
  git: {
    onChanged: (cb: (map: GitStatusMap) => void): (() => void) => {
      const listener = (_e: unknown, map: GitStatusMap): void => cb(map)
      ipcRenderer.on(IPC.GIT_CHANGED, listener)
      return () => ipcRenderer.removeListener(IPC.GIT_CHANGED, listener)
    },
    /** Commits that touched a note, newest first. */
    log: (rel: string): Promise<GitLogEntry[]> => ipcRenderer.invoke(IPC.GIT_LOG, rel),
    /** Unified diff of what one commit did to the note. */
    diffCommit: (rel: string, hash: string): Promise<string> =>
      ipcRenderer.invoke(IPC.GIT_DIFF_COMMIT, rel, hash),
    /** Unified diff of the note's uncommitted changes ('' when clean). */
    diffHead: (rel: string): Promise<string> => ipcRenderer.invoke(IPC.GIT_DIFF_HEAD, rel)
  },
  indiana: {
    copyAll: (): Promise<CopyAllResult> => ipcRenderer.invoke(IPC.INDIANA_COPY_ALL)
  }
}

contextBridge.exposeInMainWorld('api', api)

export type CasablancaApi = typeof api
