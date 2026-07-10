import { useCallback, useEffect, useState } from 'react'
import type { AgentJob, AgentJobsResult, ElicitationAction } from '@shared/domain'

const POLL_MS = 1_000

function jobLabel(job: AgentJob): string {
  if (job.markers.length > 1) return `Group · ${job.markers.length} markers`
  return job.markers[0]?.split('/').pop() ?? 'Agent'
}

function QuestionPopover({
  job,
  onAnswer
}: {
  job: AgentJob
  onAnswer: (action: ElicitationAction, answer?: string) => Promise<void>
}) {
  const [answer, setAnswer] = useState('')
  const [sending, setSending] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const question = job.question
  if (!question) return null

  const submit = async (action: ElicitationAction): Promise<void> => {
    setSending(true)
    setError(null)
    try {
      await onAnswer(action, action === 'accept' ? answer : undefined)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSending(false)
    }
  }

  return (
    <section className="absolute right-0 top-7 z-50 w-80 rounded border border-pane-border bg-pane p-3 text-left text-text-body shadow-xl">
      <p className="mb-2 text-xs font-medium text-text-strong">Agent needs an answer</p>
      <p className="mb-3 whitespace-pre-wrap text-xs leading-5">{question.message}</p>
      <textarea
        value={answer}
        onChange={(event) => setAnswer(event.target.value)}
        placeholder="Type an answer…"
        disabled={sending}
        className="mb-2 min-h-16 w-full resize-y rounded border border-pane-border bg-transparent px-2 py-1 text-xs outline-none focus:border-project disabled:opacity-50"
      />
      {error && <p className="mb-2 text-[11px] text-git-deleted">{error}</p>}
      <div className="flex justify-end gap-2 text-xs">
        <button
          onClick={() => void submit('cancel')}
          disabled={sending}
          className="rounded px-2 py-1 hover:bg-pane-hover disabled:opacity-50"
        >
          Cancel
        </button>
        <button
          onClick={() => void submit('decline')}
          disabled={sending}
          className="rounded px-2 py-1 hover:bg-pane-hover disabled:opacity-50"
        >
          Decline
        </button>
        <button
          onClick={() => void submit('accept')}
          disabled={sending}
          className="rounded bg-project px-2 py-1 text-white disabled:opacity-50"
        >
          {sending ? 'Sending…' : 'Send'}
        </button>
      </div>
    </section>
  )
}

/**
 * A slim project identity bar plus a projection of daemon-owned ACP turns.
 * The daemon outlives this renderer; polling lets a reopened Casablanca regain
 * the current processes without owning their lifecycle.
 */
export function TopBar({ name }: { name: string }) {
  const [state, setState] = useState<AgentJobsResult>({ online: false, jobs: [] })
  const [openJobId, setOpenJobId] = useState<string | null>(null)

  const refresh = useCallback(async (): Promise<void> => {
    const next = await window.api.indiana.jobs()
    setState(next)
    setOpenJobId((open) => (open && !next.jobs.some((job) => job.id === open) ? null : open))
  }, [])

  useEffect(() => {
    void refresh()
    const timer = window.setInterval(() => void refresh(), POLL_MS)
    return () => window.clearInterval(timer)
  }, [refresh])

  const answer = useCallback(
    async (job: AgentJob, action: ElicitationAction, value?: string): Promise<void> => {
      const result = await window.api.indiana.answerJob(job.id, action, value)
      if (!result.accepted) throw new Error('This agent question is no longer waiting')
      setOpenJobId(null)
      await refresh()
    },
    [refresh]
  )

  return (
    <header className="flex h-8 shrink-0 select-none items-center gap-2 bg-project px-3 text-xs font-medium text-white/95">
      <span className="max-w-48 shrink-0 truncate drop-shadow-sm">{name}</span>
      <div className="flex min-w-0 flex-1 items-center gap-1">
        {state.jobs.slice(0, 3).map((job) => {
          const waiting = job.state === 'awaiting_input'
          const open = openJobId === job.id
          return (
            <span key={job.id} className="relative">
              <button
                onClick={() => waiting && setOpenJobId(open ? null : job.id)}
                title={waiting ? 'Agent needs an answer' : 'Agent is working'}
                className={`flex max-w-44 items-center gap-1 rounded px-1.5 py-0.5 text-[11px] ${
                  waiting ? 'bg-white/25' : 'bg-black/15'
                }`}
              >
                <span aria-hidden className={waiting ? '' : 'animate-spin'}>
                  {waiting ? '?' : '◌'}
                </span>
                <span className="truncate">{jobLabel(job)}</span>
              </button>
              {open && <QuestionPopover job={job} onAnswer={(action, value) => answer(job, action, value)} />}
            </span>
          )
        })}
        {state.jobs.length > 3 && <span className="rounded bg-black/15 px-1.5 py-0.5">+{state.jobs.length - 3}</span>}
      </div>
      <span
        title={state.online ? 'Indiana daemon online' : 'Indiana daemon offline'}
        className={`h-2 w-2 shrink-0 rounded-full ${state.online ? 'bg-green-300' : 'bg-white/35'}`}
      />
    </header>
  )
}
