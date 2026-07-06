import { useState } from 'react'
import type { useVault } from '../storage/useVault'
import { FileTree } from './FileTree'

type Vault = ReturnType<typeof useVault>

export function FolderPane({ vault }: { vault: Vault }) {
  const { tree, vaultState, activeNote, openNote, createNote } = vault
  const [creatingIn, setCreatingIn] = useState<string | null>(null)
  const [newName, setNewName] = useState('')

  const rootName =
    vaultState.status === 'ready' ? vaultState.rootPath.split('/').pop() ?? 'Vault' : 'Vault'

  const submitNew = async (dirRel: string): Promise<void> => {
    const name = newName.trim()
    if (name) await createNote(dirRel, name)
    setNewName('')
    setCreatingIn(null)
  }

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-pane-border px-3 py-2">
        <span className="truncate text-xs font-semibold uppercase tracking-wide text-text-muted">
          {rootName}
        </span>
        <button
          title="New note in vault root"
          onClick={() => {
            setCreatingIn('')
            setNewName('')
          }}
          className="rounded px-1.5 text-text-muted hover:bg-pane-hover hover:text-text-strong"
        >
          +
        </button>
      </header>

      <div className="flex-1 overflow-auto px-2 py-2 text-sm">
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
            vaultKey={vaultState.rootPath}
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
