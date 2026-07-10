import { useEffect } from 'react'
import type { useVault } from '../storage/useVault'
import { FolderPane } from '../folder-pane/FolderPane'
import { EditorPane } from '../editor/EditorPane'
import { EmptyState } from './EmptyState'
import { TopBar } from './TopBar'

type Vault = ReturnType<typeof useVault>

/** The application shell: project top bar, then folder pane | editor pane. */
export function Shell({ vault }: { vault: Vault }) {
  const { vaultState, projects } = vault

  // Drive the per-project identity color used by the top bar and the switcher dot.
  useEffect(() => {
    if (vaultState.status === 'ready') {
      document.documentElement.style.setProperty('--project-color', vaultState.color)
    }
  }, [vaultState])

  if (vaultState.status === 'unset') {
    return <EmptyState onChoose={vault.addProject} />
  }

  const activeName = projects.find((p) => p.active)?.name ?? ''

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden">
      <TopBar name={activeName} />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <aside className="w-72 shrink-0 border-r border-pane-border bg-pane">
          <FolderPane vault={vault} />
        </aside>
        <main className="flex-1 overflow-hidden">
          <EditorPane vault={vault} />
        </main>
      </div>
    </div>
  )
}
