import { useCallback, useEffect, useRef, useState } from 'react'
import { AgentIndicators } from './agents/AgentIndicators'
import { useAgentJobs } from './agents/useAgentJobs'
import { GroupButtons } from './GroupButtons'
import { StageControls } from './stage/StageControls'
import { StageIconButton } from './stage/StageIconButton'
import type { StagePanelId } from './stage/stage-panel'

type CopyStatus = { kind: 'idle' } | { kind: 'busy' } | { kind: 'done'; ok: boolean; message: string }

const COPY_STATUS_MS = 4000

function CopyIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
      <rect
        x="5.5"
        y="5.5"
        width="7"
        height="8"
        rx="1"
        stroke="currentColor"
        strokeWidth="1.3"
      />
      <path
        d="M3.5 10.5V3.5a1 1 0 011-1h6"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
    </svg>
  )
}

/**
 * Vault-wide Copy all: runs `indiana copy` for the vault root. Lives with the
 * other top-right chrome icons — the editor never compiles anything itself.
 */
function CopyAllButton() {
  const [status, setStatus] = useState<CopyStatus>({ kind: 'idle' })
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current)
    },
    []
  )

  const copyAll = useCallback(async () => {
    if (timer.current) clearTimeout(timer.current)
    setStatus({ kind: 'busy' })
    const res = await window.api.indiana.copyAll().catch((err: unknown) => ({
      ok: false,
      message: err instanceof Error ? err.message : String(err)
    }))
    setStatus({ kind: 'done', ...res })
    timer.current = setTimeout(() => setStatus({ kind: 'idle' }), COPY_STATUS_MS)
  }, [])

  const title =
    status.kind === 'busy'
      ? 'Copying…'
      : status.kind === 'done'
        ? status.message
        : 'Copy all markers to clipboard'

  return (
    <StageIconButton
      title={title}
      selected={status.kind === 'done' && status.ok}
      disabled={status.kind === 'busy'}
      onClick={() => void copyAll()}
    >
      <CopyIcon />
    </StageIconButton>
  )
}

/**
 * Slim project identity bar plus stage chrome: the centered batch cluster
 * (numeric groups, agent personas, Copy all), live agent indicators, and the
 * right-panel icon controls. TopBar composes; it does not own panel content
 * or agent lifecycle.
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
      <div className="flex min-w-0 flex-1 items-center justify-center gap-1">
        <GroupButtons jobs={jobs} onAnswer={answer} />
        <CopyAllButton />
      </div>
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
