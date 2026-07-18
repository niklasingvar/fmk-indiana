import { AgentIndicators } from './agents/AgentIndicators'
import { useAgentJobs } from './agents/useAgentJobs'
import { StageControls } from './stage/StageControls'
import type { StagePanelId } from './stage/stage-panel'

/**
 * Slim project identity bar plus stage chrome: live agent indicators and
 * the right-panel icon controls. TopBar composes; it does not own panel
 * content or agent lifecycle.
 */
export function TopBar({
  name,
  selected,
  available,
  onTogglePanel
}: {
  name: string
  selected: StagePanelId | null
  available: Record<StagePanelId, boolean>
  onTogglePanel: (id: StagePanelId) => void
}) {
  const { online, jobs, openJobId, setOpenJobId, answer } = useAgentJobs()

  return (
    <header className="flex h-8 shrink-0 select-none items-center gap-2 bg-project px-3 text-xs font-medium text-white/95">
      <span className="max-w-48 shrink-0 truncate drop-shadow-sm">{name}</span>
      <div className="min-w-0 flex-1" />
      <AgentIndicators
        jobs={jobs}
        openJobId={openJobId}
        onOpenJob={setOpenJobId}
        onAnswer={answer}
      />
      <StageControls selected={selected} available={available} onToggle={onTogglePanel} />
      <span
        title={online ? 'Indiana daemon online' : 'Indiana daemon offline'}
        className={`h-2 w-2 shrink-0 rounded-full ${online ? 'bg-green-300' : 'bg-white/35'}`}
      />
    </header>
  )
}
