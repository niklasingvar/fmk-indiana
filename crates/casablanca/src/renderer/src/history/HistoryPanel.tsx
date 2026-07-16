import { useCallback, useEffect, useState } from 'react'
import type { GitLogEntry } from '@shared/domain'
import { parseUnifiedDiff, type DiffLine } from '@shared/diff'

type Selection = { kind: 'working' } | { kind: 'commit'; hash: string }

function timeAgo(ts: number): string {
  const s = Math.max(0, Math.floor((Date.now() - ts) / 1000))
  if (s < 60) return 'just now'
  if (s < 3600) return `${Math.floor(s / 60)}m ago`
  if (s < 86400) return `${Math.floor(s / 3600)}h ago`
  if (s < 7 * 86400) return `${Math.floor(s / 86400)}d ago`
  return new Date(ts).toLocaleDateString()
}

const LINE_STYLE: Record<DiffLine['kind'], string> = {
  hunk: 'bg-code-bg text-text-muted',
  added: 'bg-git-new/10 text-git-new',
  removed: 'bg-git-deleted/10 text-git-deleted',
  context: 'text-text-body'
}

const LINE_MARKER: Record<DiffLine['kind'], string> = {
  hunk: '',
  added: '+',
  removed: '-',
  context: ' '
}

function DiffView({ lines, emptyText }: { lines: DiffLine[]; emptyText: string }) {
  if (lines.length === 0) {
    return <div className="flex flex-1 items-center justify-center p-4 text-xs text-text-muted">{emptyText}</div>
  }
  return (
    <div className="flex-1 overflow-auto py-2 font-mono text-xs leading-5">
      {lines.map((line, i) => (
        <div key={i} className={`whitespace-pre-wrap break-words px-3 ${LINE_STYLE[line.kind]}`}>
          {line.kind === 'hunk' ? line.text : `${LINE_MARKER[line.kind]} ${line.text}`}
        </div>
      ))}
    </div>
  )
}

function EntryRow({
  active,
  onClick,
  title,
  subtitle,
  dotClass
}: {
  active: boolean
  onClick: () => void
  title: string
  subtitle: string
  dotClass?: string
}) {
  return (
    <button
      onClick={onClick}
      className={`block w-full px-3 py-1.5 text-left ${active ? 'bg-pane-active' : 'hover:bg-pane-hover'}`}
    >
      <span className="flex items-center gap-1.5">
        {dotClass && <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${dotClass}`} />}
        <span className="truncate text-xs text-text-strong" title={title}>
          {title}
        </span>
      </span>
      <span className="block truncate text-[11px] text-text-muted">{subtitle}</span>
    </button>
  )
}

/**
 * Read-only git history for the active note: the commits that touched it
 * (one per Indiana loop task, by the system prompt's commit convention) plus a
 * "Current changes" entry for the uncommitted working-tree diff. Selecting
 * an entry shows a unified source diff. Casablanca never writes history
 * here — committing is the coding agent's job.
 */
export function HistoryPanel({ notePath }: { notePath: string }) {
  const [entries, setEntries] = useState<GitLogEntry[]>([])
  const [workingDiff, setWorkingDiff] = useState('')
  const [selected, setSelected] = useState<Selection | null>(null)
  const [commitDiff, setCommitDiff] = useState('')

  const refresh = useCallback(async () => {
    const [log, head] = await Promise.all([
      window.api.git.log(notePath).catch(() => [] as GitLogEntry[]),
      window.api.git.diffHead(notePath).catch(() => '')
    ])
    setEntries(log)
    setWorkingDiff(head)
    // Keep a valid selection: an explicit commit stays; otherwise prefer the
    // live diff, then the newest commit.
    setSelected((prev) => {
      if (prev?.kind === 'commit' && log.some((e) => e.hash === prev.hash)) return prev
      if (head !== '') return { kind: 'working' }
      return log.length > 0 ? { kind: 'commit', hash: log[0].hash } : null
    })
  }, [notePath])

  // Initial load + follow every git push (autosaves, watcher, loop commits).
  useEffect(() => {
    void refresh()
    return window.api.git.onChanged(() => void refresh())
  }, [refresh])

  // Fetch the patch when a commit is selected.
  useEffect(() => {
    if (selected?.kind !== 'commit') return
    let cancelled = false
    void window.api.git
      .diffCommit(notePath, selected.hash)
      .catch(() => '')
      .then((d) => {
        if (!cancelled) setCommitDiff(d)
      })
    return () => {
      cancelled = true
    }
  }, [selected, notePath])

  if (entries.length === 0 && workingDiff === '') {
    return (
      <div className="flex items-center justify-center p-6 text-center text-xs text-text-muted">
        No history yet. Changes appear here once the file is edited or a task commits.
      </div>
    )
  }

  const diffText = selected?.kind === 'working' ? workingDiff : selected ? commitDiff : ''
  const emptyText =
    selected?.kind === 'working' ? 'No uncommitted changes.' : 'No changes to this file in that commit.'

  return (
    <div className="flex h-full flex-col">
      <div className="max-h-[45%] shrink-0 overflow-auto border-b border-pane-border py-1">
        {workingDiff !== '' && (
          <EntryRow
            active={selected?.kind === 'working'}
            onClick={() => setSelected({ kind: 'working' })}
            title="Current changes"
            subtitle="not committed yet"
            dotClass="bg-git-modified"
          />
        )}
        {entries.map((e) => (
          <EntryRow
            key={e.hash}
            active={selected?.kind === 'commit' && selected.hash === e.hash}
            onClick={() => setSelected({ kind: 'commit', hash: e.hash })}
            title={e.subject}
            subtitle={`${timeAgo(e.timestamp)} · ${e.hash.slice(0, 7)}`}
          />
        ))}
      </div>
      <DiffView lines={parseUnifiedDiff(diffText)} emptyText={emptyText} />
    </div>
  )
}
