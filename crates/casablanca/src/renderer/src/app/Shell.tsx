import { useEffect, useMemo, useState } from 'react'
import { isHtmlPath } from '@shared/annotation-line'
import type { useVault } from '../storage/useVault'
import { FolderPane } from '../folder-pane/FolderPane'
import { EditorPane } from '../editor/EditorPane'
import { EmptyState } from './EmptyState'
import { TopBar } from './TopBar'
import { StagePanel } from './stage/StagePanel'
import type { StagePanelId } from './stage/stage-panel'

type Vault = ReturnType<typeof useVault>

/** The application shell: project top bar, then folder pane | editor | stage panel. */
export function Shell({ vault }: { vault: Vault }) {
  const { vaultState, projects, activeNote, draft } = vault
  const [selected, setSelected] = useState<StagePanelId | null>(null)

  // Drive the per-project identity color used by the top bar and the switcher dot.
  useEffect(() => {
    if (vaultState.status === 'ready') {
      document.documentElement.style.setProperty('--project-color', vaultState.color)
    }
  }, [vaultState])

  const isMarkdownNote = activeNote !== null && !isHtmlPath(activeNote.path)
  const hasFrontmatter = draft?.frontmatter !== null && draft?.frontmatter !== undefined

  const available = useMemo(
    (): Record<StagePanelId, boolean> => ({
      properties: isMarkdownNote && hasFrontmatter,
      markers: true,
      tasks: true,
      runs: true,
      history: isMarkdownNote
    }),
    [isMarkdownNote, hasFrontmatter]
  )

  // Close a note-scoped panel when the open note can no longer host it.
  useEffect(() => {
    setSelected((current) => {
      if (current === null) return null
      return available[current] ? current : null
    })
  }, [available])

  if (vaultState.status === 'unset') {
    return <EmptyState onChoose={vault.addProject} />
  }

  const activeName = projects.find((p) => p.active)?.name ?? ''

  const togglePanel = (id: StagePanelId): void => {
    setSelected((current) => (current === id ? null : id))
  }

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden">
      <TopBar
        name={activeName}
        selected={selected}
        available={available}
        onTogglePanel={togglePanel}
      />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        <aside className="w-60 shrink-0 border-r border-pane-border bg-pane">
          <FolderPane vault={vault} />
        </aside>
        <main className="flex-1 overflow-hidden">
          <EditorPane vault={vault} />
        </main>
        {selected !== null && <StagePanel selected={selected} vault={vault} />}
      </div>
    </div>
  )
}
