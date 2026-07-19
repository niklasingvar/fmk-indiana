/**
 * IPC channel names and payloads. Shared between main and renderer so the
 * contract is a single source of truth (kept here, in the shared layer).
 *
 * The preload bridge exposes a typed `window.api` surface built from these.
 */

export const IPC = {
  VAULT_GET: 'vault:get',

  PROJECTS_LIST: 'projects:list',
  PROJECTS_ADD: 'projects:add',
  PROJECTS_SWITCH: 'projects:switch',
  PROJECTS_SET_COLOR: 'projects:set-color',
  PROJECTS_REMOVE: 'projects:remove',

  TREE_READ: 'tree:read',
  NOTE_READ: 'note:read',
  NOTE_WRITE: 'note:write',
  NOTE_CREATE: 'note:create',
  ENTRY_DELETE: 'entry:delete',
  ENTRY_REVEAL: 'entry:reveal',

  TREE_CHANGED: 'tree:changed',
  NOTE_CHANGED: 'note:changed',
  PREVIEW_CHANGED: 'preview:changed',
  GIT_CHANGED: 'git:changed',

  GIT_LOG: 'git:log',
  GIT_DIFF_COMMIT: 'git:diff-commit',
  GIT_DIFF_HEAD: 'git:diff-head',

  ANNOTATION_APPEND: 'annotation:append',

  INDIANA_COPY_ALL: 'indiana:copy-all',
  INDIANA_COPY_GROUP: 'indiana:copy-group',
  INDIANA_COPY_AGENT: 'indiana:copy-agent',
  INDIANA_RUN_GROUP: 'indiana:run-group',
  INDIANA_RUN_AGENT: 'indiana:run-agent',
  INDIANA_AGENTS: 'indiana:agents',
  INDIANA_MARKERS: 'indiana:markers',
  INDIANA_JOBS: 'indiana:jobs',
  INDIANA_ANSWER_JOB: 'indiana:answer-job',
  INDIANA_JOB_TRANSCRIPT: 'indiana:job-transcript',

  COS_TASKS: 'cos:tasks',
  COS_LOG: 'cos:log',
  COS_RUNS: 'cos:runs',
  COS_RUN_READ: 'cos:run-read'
} as const

export type IpcChannel = (typeof IPC)[keyof typeof IPC]
