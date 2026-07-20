import { useCallback, useEffect, useState } from 'react'
import type { AgentRun } from '@shared/domain'

const RUNS_PREFIX = '.indiana/chief-of-staff/runs/'

const OUTCOME_DOT: Record<string, string> = {
  done: 'bg-git-new',
  failed: 'bg-git-deleted'
}

/** `in 1234 out 567 tok · 0.1234 USD` — display-only; data comes structured. */
function usageLine(run: AgentRun): string {
  const parts: string[] = []
  if (run.tokensIn !== undefined || run.tokensOut !== undefined) {
    parts.push(`in ${run.tokensIn ?? 0} out ${run.tokensOut ?? 0} tok`)
  }
  if (run.cost !== undefined) {
    parts.push(`${run.cost.toFixed(4)} ${run.currency ?? 'USD'}`)
  }
  return parts.join(' · ')
}

/** The transcript body — everything after the frontmatter fence. */
function recordBody(text: string): string {
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
  return (
    <button
      onClick={onClick}
      title={run.detail ? `${run.outcome}: ${run.detail}` : run.outcome}
      className={`block w-full px-3 py-1.5 text-left ${active ? 'bg-pane-active' : 'hover:bg-pane-hover'}`}
    >
      <span className="flex items-center gap-1.5">
        <span
          className={`h-1.5 w-1.5 shrink-0 rounded-full ${OUTCOME_DOT[run.outcome] ?? 'bg-white/30'}`}
        />
        <span className="truncate text-xs text-text-strong">{run.job}</span>
        <span className="ml-auto shrink-0 text-[11px] text-text-muted">{run.started}</span>
      </span>
      <span className="block truncate pl-3 text-[11px] text-text-muted">
        {usageLine(run) || run.detail || run.outcome}
      </span>
    </button>
  )
}

function RunDetail({ run, body }: { run: AgentRun; body: string }) {
  const facts: [string, string | undefined][] = [
    ['outcome', run.detail ? `${run.outcome} (${run.detail})` : run.outcome],
    ['ran', run.ended && run.started ? `${run.started} → ${run.ended} UTC` : run.started],
    ['markers', run.markers?.join(', ')],
    ['tokens', usageLine(run) || undefined],
    [
      'context',
      run.contextUsed !== undefined && run.contextSize !== undefined
        ? `${run.contextUsed} of ${run.contextSize} tokens used`
        : undefined
    ]
  ]
  return (
    <div className="flex-1 overflow-auto p-3">
      <div className="mb-2 border-b border-pane-border pb-2 text-[11px] text-text-muted">
        {facts
          .filter(([, value]) => value)
          .map(([name, value]) => (
            <div key={name} className="truncate" title={value}>
              <span className="text-text-body">{name}</span> {value}
            </div>
          ))}
      </div>
      <div className="whitespace-pre-wrap break-words font-mono text-xs leading-5 text-text-body">
        {body}
      </div>
    </div>
  )
}

/**
 * Agent-run history: the durable audit records the daemon writes under
 * `.indiana/chief-of-staff/runs/` (COS_PRD.md) — one per turn. The list and
 * summary facts come from `indiana runs --json` (core computes, faces
 * render); only the selected record's transcript body is read raw.
 */
export function RunsPanel() {
  const [available, setAvailable] = useState(true)
  const [runs, setRuns] = useState<AgentRun[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [record, setRecord] = useState('')

  const refresh = useCallback(async (): Promise<void> => {
    const result = await window.api.cos.runs()
    setAvailable(result.available)
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
        if (!cancelled) setRecord(text)
      })
    return () => {
      cancelled = true
    }
  }, [selected])

  if (!available) {
    return (
      <div className="p-4 text-xs text-text-muted">
        Agent runs unavailable — install or update indiana (brew install
        niklasingvar/fmk-indiana/indiana)
      </div>
    )
  }

  if (runs.length === 0) {
    return (
      <div className="flex items-center justify-center p-6 text-center text-xs text-text-muted">
        No agent runs yet. Records appear here after a turn finishes.
      </div>
    )
  }

  const selectedRun = runs.find((run) => run.file === selected)

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
      {selectedRun ? (
        <RunDetail run={selectedRun} body={recordBody(record)} />
      ) : (
        <div className="flex-1 p-3 text-xs text-text-muted">Select a run to see its record.</div>
      )}
    </div>
  )
}
