import { useState } from 'react'
import type { useVault } from '../storage/useVault'
import type { FlatTreeNode } from '@shared/flatten-tree'
import { FileTree } from './FileTree'
import { ProjectSwitcher } from './ProjectSwitcher'

type Vault = ReturnType<typeof useVault>

export function FolderPane({ vault }: { vault: Vault }) {
  const { tree, gitStatus, vaultState, activeNote, openNote, createNote, removeEntry, revealEntry } = vault
  const [creatingIn, setCreatingIn] = useState<string | null>(null)
  const [newName, setNewName] = useState('')

  const submitNew = async (dirRel: string): Promise<void> => {
    const name = newName.trim()
    if (name) await createNote(dirRel, name)
    setNewName('')
    setCreatingIn(null)
  }

  const requestDelete = async (node: FlatTreeNode): Promise<void> => {
    const prompt =
      node.type === 'folder'
        ? `Move "${node.name}" and all of its contents to the Trash?`
        : `Move "${node.name}" to the Trash?`
    if (!window.confirm(prompt)) return

    try {
      await removeEntry(node.path)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      window.alert(`Could not move "${node.name}" to the Trash.\n\n${message}`)
    }
  }

  const requestReveal = async (node: FlatTreeNode): Promise<void> => {
    try {
      await revealEntry(node.path)
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      window.alert(`Could not reveal "${node.name}" in Finder.\n\n${message}`)
    }
  }

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-1 border-b border-pane-border px-2 py-1.5">
        <ProjectSwitcher vault={vault} />
        <button
          title="New note in project root"
          onClick={() => {
            setCreatingIn('')
            setNewName('')
          }}
          className="shrink-0 rounded px-1.5 py-1 text-text-muted hover:bg-pane-hover hover:text-text-strong"
        >
          +
        </button>
      </header>

      <div className="flex-1 overflow-auto px-2 py-1 text-sm">
        {creatingIn !== null && (
          <div className="px-2 py-1">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void submitNew(creatingIn)
                if (e.key === 'Escape') setCreatingIn(null)
              }}
              onBlur={() => creatingIn !== null && void submitNew(creatingIn)}
              placeholder="note name"
              className="w-full rounded border border-pane-border bg-pane-active px-2 py-1 text-sm outline-none focus:border-accent"
            />
          </div>
        )}

        {tree?.children?.length && vaultState.status === 'ready' ? (
          <FileTree
            key={vaultState.rootPath}
            tree={tree}
            activePath={activeNote?.path ?? null}
            onOpen={openNote}
            onDelete={requestDelete}
            onReveal={requestReveal}
            vaultKey={vaultState.rootPath}
            gitStatus={gitStatus}
          />
        ) : (
          <p className="px-3 py-4 text-xs text-text-muted">
            No notes yet. Create one with +.
          </p>
        )}
      </div>
    </div>
  )
}
