import { useRef, useState } from 'react'
import type { useVault } from '../storage/useVault'
import type { FlatTreeNode } from '@shared/flatten-tree'
import { FileTree, type FileTreeHandle } from './FileTree'
import { ProjectSwitcher } from './ProjectSwitcher'

type Vault = ReturnType<typeof useVault>

export function FolderPane({ vault }: { vault: Vault }) {
  const { tree, gitStatus, vaultState, activeNote, openNote, createNote, removeEntry, revealEntry } = vault
  const [creatingIn, setCreatingIn] = useState<string | null>(null)
  const [newName, setNewName] = useState('')
  const treeRef = useRef<FileTreeHandle>(null)

  const submitNew = async (dirRel: string): Promise<void> => {
    const name = newName.trim()
    if (name) await createNote(dirRel, name)
    setNewName('')
    setCreatingIn(null)
  }

  const requestCreate = (dirRel: string): void => {
    setCreatingIn(dirRel)
    setNewName('')
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
      <header className="flex items-center gap-0.5 border-b border-pane-border px-1 py-1">
        <ProjectSwitcher vault={vault} />
        <button
          title="Collapse all folders"
          aria-label="Collapse all folders"
          onClick={() => treeRef.current?.collapseAll()}
          className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-text-muted hover:bg-pane-hover hover:text-text-strong"
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
            <path
              d="M4 12l4-4 4 4M4 8l4-4 4 4"
              stroke="currentColor"
              strokeWidth="1.3"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </button>
        <button
          title="New note in project root"
          onClick={() => requestCreate('')}
          className="shrink-0 rounded px-1 py-0.5 text-sm text-text-muted hover:bg-pane-hover hover:text-text-strong"
        >
          +
        </button>
      </header>

      <div className="flex-1 overflow-auto px-1 py-1">
        {creatingIn !== null && (
          <div className="px-1 py-0.5">
            <input
              autoFocus
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') void submitNew(creatingIn)
                if (e.key === 'Escape') setCreatingIn(null)
              }}
              onBlur={() => creatingIn !== null && void submitNew(creatingIn)}
              placeholder={creatingIn ? `New note in ${creatingIn}` : 'New note'}
              className="w-full rounded border border-pane-border bg-pane-active px-1.5 py-0.5 text-xs outline-none focus:border-accent"
            />
          </div>
        )}

        {tree?.children?.length && vaultState.status === 'ready' ? (
          <FileTree
            key={vaultState.rootPath}
            ref={treeRef}
            tree={tree}
            activePath={activeNote?.path ?? null}
            onOpen={openNote}
            onCreate={requestCreate}
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
