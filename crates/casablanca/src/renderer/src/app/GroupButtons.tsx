import { useCallback, useEffect, useRef, useState } from 'react'
import type { AgentJob, CopyAllResult, ElicitationAction } from '@shared/domain'
import { jobForBatch } from '@shared/batch-job'

import { JobFollowPopover } from './agents/JobFollowPopover'

/** One batch the top bar offers: a numeric group or a named agent persona. */
export interface Batch {
  /** `group-1` / `agent-mike` — stable render key. */
  key: string
  /** Button face: the group digit or the agent's first letter, uppercased. */
  label: string
  /** Tooltip subject: `group -1` or the agent's full name. */
  title: string
  count: number
  copy: () => Promise<CopyAllResult>
  dispatch: () => Promise<{ accepted: boolean; count: number }>
}

const REFRESH_DEBOUNCE_MS = 1000
const STATUS_MS = 4000
/** Groups 1–3 are always on offer; higher labels appear when markers use them. */
const MIN_GROUPS = 3

/**
 * The vault's batch roster: numeric groups (from a read-only scan) and agent
 * personas (from `.indiana/agents/`), with live member counts. Core computes,
 * faces render — this hook only tallies fields the scan already carries.
 */
export function useBatches(): Batch[] {
  const [groupCounts, setGroupCounts] = useState<Record<number, number>>({})
  const [agentCounts, setAgentCounts] = useState<Record<string, number>>({})
  const [agents, setAgents] = useState<string[]>([])

  const refresh = useCallback(async (): Promise<void> => {
    const [markerResult, agentResult] = await Promise.all([
      window.api.indiana.markers(),
      window.api.indiana.agents()
    ])
    const groups: Record<number, number> = {}
    const perAgent: Record<string, number> = {}
    for (const marker of markerResult.markers) {
      if (marker.group !== undefined) groups[marker.group] = (groups[marker.group] ?? 0) + 1
      if (marker.agent !== undefined) perAgent[marker.agent] = (perAgent[marker.agent] ?? 0) + 1
    }
    setGroupCounts(groups)
    setAgentCounts(perAgent)
    setAgents(agentResult.agents)
  }, [])

  const debounce = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => {
    void refresh()
    const off = window.api.tree.onChanged(() => {
      if (debounce.current) clearTimeout(debounce.current)
      debounce.current = setTimeout(() => void refresh(), REFRESH_DEBOUNCE_MS)
    })
    return () => {
      off()
      if (debounce.current) clearTimeout(debounce.current)
    }
  }, [refresh])

  const highestGroup = Math.max(MIN_GROUPS, ...Object.keys(groupCounts).map(Number))
  const groups: Batch[] = Array.from({ length: highestGroup }, (_, i) => i + 1).map((group) => ({
    key: `group-${group}`,
    label: String(group),
    title: `group -${group}`,
    count: groupCounts[group] ?? 0,
    copy: () => window.api.indiana.copyGroup(group),
    dispatch: () => window.api.indiana.runGroup(group)
  }))
  const personas: Batch[] = agents.map((agent) => ({
    key: `agent-${agent}`,
    label: agent.slice(0, 1).toUpperCase(),
    title: agent,
    count: agentCounts[agent] ?? 0,
    copy: () => window.api.indiana.copyAgent(agent),
    dispatch: () => window.api.indiana.runAgent(agent)
  }))
  return [...groups, ...personas]
}

type ButtonStatus =
  | { kind: 'idle' }
  | { kind: 'busy' }
  /** Dispatch accepted; the daemon job hasn't shown up in the 1s poll yet. */
  | { kind: 'launching' }
  | { kind: 'done'; ok: boolean; message: string }

/** How long to trust `launching` before conceding the job never appeared. */
const LAUNCH_GRACE_MS = 8000

function BatchButton({
  batch,
  job,
  menuOpen,
  onOpenMenu,
  onAnswer
}: {
  batch: Batch
  /** The live daemon job for this batch, when one is running. */
  job: AgentJob | undefined
  menuOpen: boolean
  onOpenMenu: (key: string | null) => void
  onAnswer: (job: AgentJob, action: ElicitationAction, value?: string) => Promise<void>
}) {
  const [status, setStatus] = useState<ButtonStatus>({ kind: 'idle' })
  const [followOpen, setFollowOpen] = useState(false)
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const hadJob = useRef(false)

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current)
    },
    []
  )

  const flash = (ok: boolean, message: string): void => {
    if (timer.current) clearTimeout(timer.current)
    setStatus({ kind: 'done', ok, message })
    timer.current = setTimeout(() => setStatus({ kind: 'idle' }), STATUS_MS)
  }

  // The job appearing ends `launching`; the job disappearing after a run
  // reports the turn ending — the user always sees start and end.
  useEffect(() => {
    if (job) {
      hadJob.current = true
      setStatus((s) => (s.kind === 'launching' ? { kind: 'idle' } : s))
    } else {
      setFollowOpen(false)
      if (hadJob.current) {
        hadJob.current = false
        flash(true, `Turn ended for ${batch.title} — see the Agent runs panel`)
      }
    }
  }, [job !== undefined])

  const copy = async (): Promise<void> => {
    onOpenMenu(null)
    if (timer.current) clearTimeout(timer.current)
    setStatus({ kind: 'busy' })
    const res = await batch.copy().catch((err: unknown) => ({
      ok: false,
      message: err instanceof Error ? err.message : String(err)
    }))
    flash(res.ok, res.message)
  }

  const dispatch = async (): Promise<void> => {
    onOpenMenu(null)
    if (timer.current) clearTimeout(timer.current)
    setStatus({ kind: 'busy' })
    const res = await batch
      .dispatch()
      .catch(() => ({ accepted: false, count: 0 }))
    if (!res.accepted) {
      flash(
        false,
        `Could not dispatch ${batch.title} — daemon offline, batch empty, or a turn is running`
      )
      return
    }
    // Keep spinning until the jobs poll picks the turn up (≤1s), then the
    // job itself drives the spinner. The grace timer covers a turn so fast
    // it ends between polls.
    setStatus({ kind: 'launching' })
    if (timer.current) clearTimeout(timer.current)
    timer.current = setTimeout(
      () => setStatus((s) => (s.kind === 'launching' ? { kind: 'idle' } : s)),
      LAUNCH_GRACE_MS
    )
  }

  const empty = batch.count === 0
  const spinning = status.kind === 'busy' || status.kind === 'launching' || job !== undefined
  const waiting = job?.state === 'awaiting_input'
  const title = waiting
    ? `${batch.title} — agent needs an answer, click to reply`
    : job
      ? `${batch.title} — agent running, click to follow`
      : status.kind === 'busy' || status.kind === 'launching'
        ? `Dispatching ${batch.title}…`
        : status.kind === 'done'
          ? status.message
          : empty
            ? `${batch.title} — no indianas`
            : `Copy ${batch.title} · ${batch.count} indiana(s); right-click to dispatch`

  return (
    <span className="relative">
      <button
        type="button"
        disabled={(empty && !job && status.kind !== 'launching') || status.kind === 'busy'}
        title={title}
        aria-label={title}
        aria-busy={spinning}
        onClick={() => {
          if (job) {
            setFollowOpen((open) => !open)
            return
          }
          void copy()
        }}
        onContextMenu={(e) => {
          e.preventDefault()
          if (!empty && !job && status.kind !== 'launching') {
            onOpenMenu(menuOpen ? null : batch.key)
          }
        }}
        className={`flex h-5 min-w-5 items-center justify-center rounded px-1 text-[11px] font-semibold disabled:cursor-not-allowed disabled:opacity-40 ${
          waiting || (status.kind === 'done' && status.ok)
            ? 'bg-white/25'
            : 'bg-black/15 hover:bg-black/25'
        }`}
      >
        {waiting ? (
          <span aria-hidden>?</span>
        ) : spinning ? (
          <span
            aria-hidden
            className="h-3 w-3 shrink-0 animate-spin rounded-full border-2 border-white/40 border-t-white"
          />
        ) : (
          batch.label
        )}
      </button>
      {status.kind === 'done' && (
        <span
          role="status"
          className={`absolute left-1/2 top-6 z-50 -translate-x-1/2 whitespace-nowrap rounded px-2 py-1 text-[11px] shadow-lg ${
            status.ok ? 'bg-black/80 text-white' : 'bg-git-deleted text-white'
          }`}
        >
          {status.message}
        </span>
      )}
      {menuOpen && (
        <div className="absolute left-0 top-6 z-50 min-w-36 rounded border border-black/20 bg-project py-0.5 shadow-lg">
          <button
            type="button"
            onClick={() => void dispatch()}
            className="block w-full px-2 py-1 text-left text-[11px] hover:bg-black/25"
          >
            Dispatch to agent
          </button>
        </div>
      )}
      {followOpen && job && (
        <JobFollowPopover job={job} onAnswer={(action, value) => onAnswer(job, action, value)} />
      )}
    </span>
  )
}

/**
 * The centered batch cluster: numeric groups 1…n, one button per agent
 * persona (M = mike, L = lisa, …), each graying out while it has no tagged
 * indianas. Click copies the batch (personas carry their own system prompt);
 * right-click offers a one-item dispatch menu. While a batch's turn runs the
 * button itself is the spinner and opens the follow popover — dispatch is
 * never invisible.
 */
export function GroupButtons({
  jobs,
  onAnswer
}: {
  jobs: AgentJob[]
  onAnswer: (job: AgentJob, action: ElicitationAction, value?: string) => Promise<void>
}) {
  const batches = useBatches()
  const [openMenu, setOpenMenu] = useState<string | null>(null)

  // A click anywhere else dismisses the dispatch menu.
  useEffect(() => {
    if (openMenu === null) return undefined
    const close = (): void => setOpenMenu(null)
    window.addEventListener('click', close)
    return () => window.removeEventListener('click', close)
  }, [openMenu])

  return (
    <div className="flex items-center gap-1">
      {batches.map((batch) => (
        <BatchButton
          key={batch.key}
          batch={batch}
          job={jobForBatch(jobs, batch.key)}
          menuOpen={openMenu === batch.key}
          onOpenMenu={setOpenMenu}
          onAnswer={onAnswer}
        />
      ))}
    </div>
  )
}
