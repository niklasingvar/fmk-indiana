import type { useVault } from '../storage/useVault'
import { FolderPane } from '../folder-pane/FolderPane'
import { EditorPane } from '../editor/EditorPane'
import { EmptyState } from './EmptyState'

type Vault = ReturnType<typeof useVault>

/** The application shell: folder pane | editor pane. Nothing else. */
export function Shell({ vault }: { vault: Vault }) {
  const { vaultState } = vault

  if (vaultState.status === 'unset') {
    return <EmptyState onChoose={vault.chooseVault} />
  }

  return (
    <div className="flex h-screen w-screen overflow-hidden">
      <aside className="w-72 shrink-0 border-r border-pane-border bg-pane">
        <FolderPane vault={vault} />
      </aside>
      <main className="flex-1 overflow-hidden">
        <EditorPane vault={vault} />
      </main>
    </div>
  )
}
