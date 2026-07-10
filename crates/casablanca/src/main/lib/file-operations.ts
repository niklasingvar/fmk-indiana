import { promises as fs } from 'node:fs'
import { resolve, sep } from 'node:path'
import type { VaultConfig } from '@shared/domain'

export type TrashItem = (absolutePath: string) => Promise<void>
export type ShowItemInFolder = (absolutePath: string) => void

/**
 * Resolve one vault-relative entry without allowing the operation to escape
 * the project root or target the synthetic root itself.
 */
export function resolveEntryPath(vault: VaultConfig, rel: string): string {
  if (!rel || rel.includes('\0')) throw new Error('Invalid vault entry path')

  const root = resolve(vault.rootPath)
  const absolutePath = resolve(root, rel)
  if (absolutePath === root || !absolutePath.startsWith(`${root}${sep}`)) {
    throw new Error('Entry path escapes vault')
  }
  return absolutePath
}

/** Move one existing file or folder to the operating system Trash. */
export async function deleteEntry(
  vault: VaultConfig,
  rel: string,
  trashItem: TrashItem
): Promise<void> {
  const absolutePath = resolveEntryPath(vault, rel)
  await fs.lstat(absolutePath)
  await trashItem(absolutePath)
}

/** Reveal one existing file or folder in the operating system file manager. */
export async function revealEntry(
  vault: VaultConfig,
  rel: string,
  showItemInFolder: ShowItemInFolder
): Promise<void> {
  const absolutePath = resolveEntryPath(vault, rel)
  await fs.lstat(absolutePath)
  showItemInFolder(absolutePath)
}
