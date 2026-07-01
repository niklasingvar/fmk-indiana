/**
 * IPC channel names and payloads. Shared between main and renderer so the
 * contract is a single source of truth (kept here, in the shared layer).
 *
 * The preload bridge exposes a typed `window.api` surface built from these.
 */

export const IPC = {
  VAULT_GET: 'vault:get',
  VAULT_CHOOSE: 'vault:choose',
  VAULT_SET: 'vault:set',

  TREE_READ: 'tree:read',
  NOTE_READ: 'note:read',
  NOTE_WRITE: 'note:write',
  NOTE_CREATE: 'note:create',
  NOTE_DELETE: 'note:delete',

  TREE_CHANGED: 'tree:changed'
} as const

export type IpcChannel = (typeof IPC)[keyof typeof IPC]
