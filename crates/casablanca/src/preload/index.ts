import { contextBridge, ipcRenderer } from 'electron'
import { IPC } from '@shared/ipc'
import type {
  AnnotationRequest,
  AnnotationResult,
  CopyAllResult,
  Note,
  TreeNode,
  VaultState
} from '@shared/domain'

/**
 * The renderer's only gateway to the system. Every capability the UI has is
 * declared here explicitly — nothing else is exposed. Keep this surface small.
 */
const api = {
  vault: {
    get: (): Promise<VaultState> => ipcRenderer.invoke(IPC.VAULT_GET),
    choose: (): Promise<{ status: 'ready'; rootPath: string } | null> =>
      ipcRenderer.invoke(IPC.VAULT_CHOOSE),
    set: (rootPath: string): Promise<{ status: 'ready'; rootPath: string }> =>
      ipcRenderer.invoke(IPC.VAULT_SET, rootPath)
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
    remove: (rel: string): Promise<void> => ipcRenderer.invoke(IPC.NOTE_DELETE, rel)
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
  indiana: {
    copyAll: (): Promise<CopyAllResult> => ipcRenderer.invoke(IPC.INDIANA_COPY_ALL)
  }
}

contextBridge.exposeInMainWorld('api', api)

export type CasablancaApi = typeof api
