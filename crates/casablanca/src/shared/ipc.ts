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

  TREE_CHANGED: 'tree:changed',
  PREVIEW_CHANGED: 'preview:changed',
  GIT_CHANGED: 'git:changed',

  GIT_LOG: 'git:log',
  GIT_DIFF_COMMIT: 'git:diff-commit',
  GIT_DIFF_HEAD: 'git:diff-head',

  ANNOTATION_APPEND: 'annotation:append',

  INDIANA_COPY_ALL: 'indiana:copy-all'
} as const

export type IpcChannel = (typeof IPC)[keyof typeof IPC]
