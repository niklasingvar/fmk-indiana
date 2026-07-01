import type { useVault } from '../storage/useVault'
import { LexicalEditor } from './Editor'

type Vault = ReturnType<typeof useVault>

export function EditorPane({ vault }: { vault: Vault }) {
  const { activeNote, draft, setDraft, saving } = vault

  if (!activeNote) {
    return (
      <div className="flex h-full items-center justify-center text-text-muted">
        Select a note from the left, or create a new one.
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-pane-border px-6 py-2 text-xs text-text-muted">
        <span className="truncate font-mono">{activeNote.path}</span>
        <span>{saving ? 'Saving…' : 'Saved'}</span>
      </header>
      <div className="flex-1 overflow-auto">
        <div className="mx-auto max-w-3xl px-8 py-8">
          <LexicalEditor key={activeNote.path} markdown={draft} onChange={setDraft} />
        </div>
      </div>
    </div>
  )
}
