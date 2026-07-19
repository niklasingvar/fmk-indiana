import { contextBridge, ipcRenderer } from 'electron'
import { IPC } from '@shared/ipc'
import type {
  AnnotationRequest,
  AnnotationResult,
  AgentJobsResult,
  AgentRunsResult,
  AnswerAgentJobResult,
  CopyAllResult,
  CosLogResult,
  CosTasksResult,
  DispatchResult,
  ElicitationAction,
  GitLogEntry,
  JobTranscriptResult,
  GitStatusMap,
  Note,
  Project,
  TreeNode,
  VaultAgentsResult,
  VaultMarkersResult,
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
    /** External on-disk change to a markdown note (agent edit, daemon claim). */
    onChanged: (cb: (relPath: string) => void): (() => void) => {
      const listener = (_e: unknown, relPath: string): void => cb(relPath)
      ipcRenderer.on(IPC.NOTE_CHANGED, listener)
      return () => ipcRenderer.removeListener(IPC.NOTE_CHANGED, listener)
    }
  },
  entries: {
    remove: (rel: string): Promise<void> => ipcRenderer.invoke(IPC.ENTRY_DELETE, rel),
    reveal: (rel: string): Promise<void> => ipcRenderer.invoke(IPC.ENTRY_REVEAL, rel)
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
    copyAll: (): Promise<CopyAllResult> => ipcRenderer.invoke(IPC.INDIANA_COPY_ALL),
    /** Copy one numeric batch (`-1`, `-2`, …) with the default system prompt. */
    copyGroup: (group: number): Promise<CopyAllResult> =>
      ipcRenderer.invoke(IPC.INDIANA_COPY_GROUP, group),
    /** Copy one agent persona's batch with that agent's system prompt. */
    copyAgent: (agent: string): Promise<CopyAllResult> =>
      ipcRenderer.invoke(IPC.INDIANA_COPY_AGENT, agent),
    /** Dispatch one numeric batch as a manual agent turn. */
    runGroup: (group: number): Promise<DispatchResult> =>
      ipcRenderer.invoke(IPC.INDIANA_RUN_GROUP, group),
    /** Dispatch one agent persona's batch as a manual agent turn. */
    runAgent: (agent: string): Promise<DispatchResult> =>
      ipcRenderer.invoke(IPC.INDIANA_RUN_AGENT, agent),
    /** Agent personas defined in the vault (`.indiana/agents/`). */
    agents: (): Promise<VaultAgentsResult> => ipcRenderer.invoke(IPC.INDIANA_AGENTS),
    /** Every `::` marker in the vault, from a read-only scan. */
    markers: (): Promise<VaultMarkersResult> => ipcRenderer.invoke(IPC.INDIANA_MARKERS),
    jobs: (): Promise<AgentJobsResult> => ipcRenderer.invoke(IPC.INDIANA_JOBS),
    answerJob: (
      jobId: string,
      action: ElicitationAction,
      answer?: string
    ): Promise<AnswerAgentJobResult> =>
      ipcRenderer.invoke(IPC.INDIANA_ANSWER_JOB, jobId, action, answer),
    /** A live turn's transcript from `sinceSeq` on; found:false = turn ended. */
    transcript: (jobId: string, sinceSeq: number): Promise<JobTranscriptResult> =>
      ipcRenderer.invoke(IPC.INDIANA_JOB_TRANSCRIPT, jobId, sinceSeq)
  },
  cos: {
    /** Chief of Staff tracker rows, all states (COS_PRD.md). */
    tasks: (): Promise<CosTasksResult> => ipcRenderer.invoke(IPC.COS_TASKS),
    /** Tail of the action log, oldest first. */
    log: (lines?: number): Promise<CosLogResult> => ipcRenderer.invoke(IPC.COS_LOG, lines),
    /** Durable agent-run audit records, newest first (COS_PRD.md runs/). */
    runs: (): Promise<AgentRunsResult> => ipcRenderer.invoke(IPC.COS_RUNS),
    /** Full markdown of one run record, by filename from `runs()`. */
    run: (file: string): Promise<string> => ipcRenderer.invoke(IPC.COS_RUN_READ, file)
  }
}

contextBridge.exposeInMainWorld('api', api)

export type CasablancaApi = typeof api
