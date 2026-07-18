import { useEffect, useRef, useState } from 'react'
import type { AgentJob, ElicitationAction, TranscriptEvent } from '@shared/domain'

const POLL_MS = 1_000

/**
 * Streamed chunks merge into their last transcript event daemon-side (its
 * seq stays, its text grows), so each poll re-fetches from the last seen
 * event's seq and replaces the overlap instead of appending blindly.
 */
function mergeEvents(prev: TranscriptEvent[], page: TranscriptEvent[]): TranscriptEvent[] {
  if (page.length === 0) return prev
  return [...prev.filter((event) => event.seq < page[0].seq), ...page]
}

function EventLine({ event }: { event: TranscriptEvent }) {
  switch (event.kind) {
    case 'tool':
      return <p className="my-1 text-[11px] text-text-muted">⚙ {event.text}</p>
    case 'thought':
      return <p className="my-1 text-[11px] italic text-text-muted">{event.text}</p>
    case 'question':
      return (
        <p className="my-1 whitespace-pre-wrap text-xs">
          <span className="font-medium text-text-strong">Agent asks: </span>
          {event.text}
        </p>
      )
    case 'answer':
      return (
        <p className="my-1 whitespace-pre-wrap text-xs">
          <span className="font-medium text-text-strong">You: </span>
          {event.text}
        </p>
      )
    default:
      return <p className="my-1 whitespace-pre-wrap text-xs leading-5">{event.text}</p>
  }
}

function QuestionForm({
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
      setAnswer('')
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSending(false)
    }
  }

  return (
    <div className="mt-2 border-t border-pane-border pt-2">
      <p className="mb-2 whitespace-pre-wrap text-xs font-medium text-text-strong">
        {question.message}
      </p>
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
    </div>
  )
}

/**
 * The small follow window anchored to a job chip: a live transcript of the
 * agent's turn (polled from the daemon's in-memory projection), with the
 * question form as its footer while the turn awaits input. Transcripts die
 * with the job; a finished turn shows a quiet "turn ended".
 */
export function JobFollowPopover({
  job,
  onAnswer
}: {
  job: AgentJob
  onAnswer: (action: ElicitationAction, answer?: string) => Promise<void>
}) {
  const [events, setEvents] = useState<TranscriptEvent[]>([])
  const [ended, setEnded] = useState(false)
  const eventsRef = useRef(events)
  eventsRef.current = events
  const listRef = useRef<HTMLDivElement | null>(null)
  const stickToBottom = useRef(true)

  useEffect(() => {
    let live = true
    const poll = async (): Promise<void> => {
      const seen = eventsRef.current
      const since = seen.length > 0 ? seen[seen.length - 1].seq : 0
      const page = await window.api.indiana.transcript(job.id, since)
      if (!live) return
      if (!page.found) {
        setEnded(true)
        return
      }
      setEnded(false)
      if (page.events.length > 0) setEvents((prev) => mergeEvents(prev, page.events))
    }
    void poll()
    const timer = window.setInterval(() => void poll(), POLL_MS)
    return () => {
      live = false
      window.clearInterval(timer)
    }
  }, [job.id])

  // Follow the stream unless the user scrolled up to read.
  useEffect(() => {
    const list = listRef.current
    if (list && stickToBottom.current) list.scrollTop = list.scrollHeight
  }, [events])

  return (
    <section className="absolute right-0 top-7 z-50 w-96 rounded border border-pane-border bg-pane p-3 text-left text-text-body shadow-xl">
      <div
        ref={listRef}
        onScroll={(event) => {
          const el = event.currentTarget
          stickToBottom.current = el.scrollTop + el.clientHeight >= el.scrollHeight - 8
        }}
        className="max-h-72 overflow-y-auto"
      >
        {events.length === 0 && !ended && (
          <p className="text-xs text-text-muted">Waiting for the agent…</p>
        )}
        {events.map((event) => (
          <EventLine key={event.seq} event={event} />
        ))}
        {ended && <p className="mt-1 text-[11px] text-text-muted">Turn ended.</p>}
      </div>
      {job.state === 'awaiting_input' && <QuestionForm job={job} onAnswer={onAnswer} />}
    </section>
  )
}
