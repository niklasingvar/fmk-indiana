import type { useVault } from '../../storage/useVault'
import { MarkersPanel } from '../../markers/MarkersPanel'
import { TasksPanel } from '../../cos/TasksPanel'
import { RunsPanel } from '../../cos/RunsPanel'
import { FrontmatterPanel } from '../../editor/FrontmatterPanel'
import { HistoryPanel } from '../../history/HistoryPanel'
import type { StagePanelId } from './stage-panel'

type Vault = ReturnType<typeof useVault>

/**
 * The one shared right-side stage slot. Maps a selected panel identity onto
 * the existing content panels; vault reaches content panels only here.
 */
export function StagePanel({
  selected,
  vault
}: {
  selected: StagePanelId
  vault: Vault
}) {
  const { activeNote, draft, noteVersion, setDraftFrontmatter } = vault

  if (selected === 'markers') {
    return (
      <aside className="w-80 shrink-0 border-l border-pane-border bg-pane">
        <MarkersPanel vault={vault} />
      </aside>
    )
  }

  if (selected === 'tasks') {
    return (
      <aside className="w-80 shrink-0 border-l border-pane-border bg-pane">
        <TasksPanel vault={vault} />
      </aside>
    )
  }

  if (selected === 'runs') {
    return (
      <aside className="w-80 shrink-0 overflow-hidden border-l border-pane-border bg-pane">
        <RunsPanel />
      </aside>
    )
  }

  if (selected === 'properties') {
    if (!activeNote || draft?.frontmatter === null || draft?.frontmatter === undefined) return null
    return (
      <aside className="w-80 shrink-0 overflow-hidden border-l border-pane-border bg-pane">
        <FrontmatterPanel
          key={`${activeNote.path}:${noteVersion}`}
          frontmatter={draft.frontmatter}
          onChange={setDraftFrontmatter}
        />
      </aside>
    )
  }

  if (!activeNote) return null
  return (
    <aside className="w-80 shrink-0 overflow-hidden border-l border-pane-border bg-pane">
      <HistoryPanel key={activeNote.path} notePath={activeNote.path} />
    </aside>
  )
}
