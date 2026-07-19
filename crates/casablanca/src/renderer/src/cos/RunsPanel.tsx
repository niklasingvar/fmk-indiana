import { useCallback, useEffect, useState } from 'react'
import type { AgentRun } from '@shared/domain'

const RUNS_PREFIX = '.indiana/chief-of-staff/runs/'

const OUTCOME_DOT: Record<AgentRun['outcome'], string> = {
  done: 'bg-git-new',
  failed: 'bg-git-deleted',
  unknown: 'bg-white/30'
}

/** Drop the frontmatter fence — audit prose, not metadata, in the viewer. */
function stripFrontmatter(text: string): string {
  if (!text.startsWith('---\n')) return text
  const end = text.indexOf('\n---\n', 4)
  return end < 0 ? text : text.slice(end + 5).replace(/^\n+/, '')
}

function RunRow({
  run,
  active,
  onClick
}: {
  run: AgentRun
  active: boolean
  onClick: () => void
}) {
  const usage = [run.tokens, run.cost].filter(Boolean).join(' · ')
  return (
    <button
      onClick={onClick}
      title={run.detail ? `${run.outcome}: ${run.detail}` : run.outcome}
      className={`block w-full px-3 py-1.5 text-left ${active ? 'bg-pane-active' : 'hover:bg-pane-hover'}`}
    >
      <span className="flex items-center gap-1.5">
        <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${OUTCOME_DOT[run.outcome]}`} />
        <span className="truncate text-xs text-text-strong">{run.jobId}</span>
        <span className="ml-auto shrink-0 text-[11px] text-text-muted">{run.started}</span>
      </span>
      <span className="block truncate pl-3 text-[11px] text-text-muted">
        {usage || run.detail || run.outcome}
      </span>
    </button>
  )
}

/**
 * Agent-run history: the durable audit records the daemon writes under
 * `.indiana/chief-of-staff/runs/` (COS_PRD.md) — one per turn, with outcome,
 * transcript, and token usage. The list is the index; selecting a run shows
 * the full record. Read-only: Casablanca renders what the daemon recorded.
 */
export function RunsPanel() {
  const [runs, setRuns] = useState<AgentRun[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [record, setRecord] = useState('')

  const refresh = useCallback(async (): Promise<void> => {
    const result = await window.api.cos.runs()
    setRuns(result.runs)
    setSelected((prev) =>
      prev !== null && result.runs.some((run) => run.file === prev)
        ? prev
        : (result.runs[0]?.file ?? null)
    )
  }, [])

  // Refresh when a new record lands — the watcher pushes per-path note
  // events for .md files under the watched .indiana/.
  useEffect(() => {
    void refresh()
    return window.api.notes.onChanged((rel) => {
      if (rel.startsWith(RUNS_PREFIX)) void refresh()
    })
  }, [refresh])

  useEffect(() => {
    if (selected === null) {
      setRecord('')
      return
    }
    let cancelled = false
    void window.api.cos
      .run(selected)
      .catch(() => '')
      .then((text) => {
        if (!cancelled) setRecord(stripFrontmatter(text))
      })
    return () => {
      cancelled = true
    }
  }, [selected])

  if (runs.length === 0) {
    return (
      <div className="flex items-center justify-center p-6 text-center text-xs text-text-muted">
        No agent runs yet. Records appear here after a turn finishes.
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col">
      <div className="max-h-[45%] shrink-0 overflow-auto border-b border-pane-border py-1">
        {runs.map((run) => (
          <RunRow
            key={run.file}
            run={run}
            active={selected === run.file}
            onClick={() => setSelected(run.file)}
          />
        ))}
      </div>
      <div className="flex-1 overflow-auto whitespace-pre-wrap break-words p-3 font-mono text-xs leading-5 text-text-body">
        {record || 'Select a run to see its record.'}
      </div>
    </div>
  )
}
