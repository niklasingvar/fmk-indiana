import type { ReactElement } from 'react'
import { StageIconButton } from './StageIconButton'
import { STAGE_PANELS, type StagePanelId } from './stage-panel'

function PropertiesIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
      <path
        d="M3 3.5h10v9H3zM5.5 6h5M5.5 8.5h5M5.5 11h3"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function MarkersIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
      <circle cx="2.6" cy="4.5" r="0.9" fill="currentColor" />
      <circle cx="5.1" cy="4.5" r="0.9" fill="currentColor" />
      <circle cx="2.6" cy="8" r="0.9" fill="currentColor" />
      <circle cx="5.1" cy="8" r="0.9" fill="currentColor" />
      <circle cx="2.6" cy="11.5" r="0.9" fill="currentColor" />
      <circle cx="5.1" cy="11.5" r="0.9" fill="currentColor" />
      <path
        d="M7.8 4.5h5.7M7.8 8h5.7M7.8 11.5h3.7"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
    </svg>
  )
}

function TasksIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
      <path
        d="M3.5 4.5h9M3.5 8h9M3.5 11.5h6"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
      <path
        d="M2 4.5l.8.8L4.2 3.9M2 8l.8.8L4.2 7.4"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function RunsIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
      <path
        d="M8.8 2L4 9h3.2L7.2 14 12 7H8.8L8.8 2z"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function HistoryIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 16 16" fill="none" aria-hidden>
      <path
        d="M8 3a5 5 0 11-4.3 2.5"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
      />
      <path
        d="M3.2 3.2v2.6h2.6M8 5.5V8l1.8 1.2"
        stroke="currentColor"
        strokeWidth="1.3"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

const ICONS: Record<StagePanelId, () => ReactElement> = {
  properties: PropertiesIcon,
  markers: MarkersIcon,
  tasks: TasksIcon,
  runs: RunsIcon,
  history: HistoryIcon
}

/**
 * Top-right stage panel icons. Presentational — selection and availability
 * come from Shell; this file never reads vault or daemon state.
 */
export function StageControls({
  selected,
  available,
  onToggle
}: {
  selected: StagePanelId | null
  available: Record<StagePanelId, boolean>
  onToggle: (id: StagePanelId) => void
}) {
  return (
    <div className="flex items-center gap-1">
      {STAGE_PANELS.map((panel) => {
        const Icon = ICONS[panel.id]
        const isAvailable = available[panel.id]
        const isSelected = selected === panel.id
        return (
          <StageIconButton
            key={panel.id}
            title={
              !isAvailable
                ? `${panel.title} unavailable`
                : isSelected
                  ? `Hide ${panel.title}`
                  : `Show ${panel.title}`
            }
            selected={isSelected}
            disabled={!isAvailable}
            onClick={() => onToggle(panel.id)}
          >
            <Icon />
          </StageIconButton>
        )
      })}
    </div>
  )
}
