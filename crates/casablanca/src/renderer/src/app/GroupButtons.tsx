import { useCallback, useEffect, useRef, useState } from 'react'
import type { CopyAllResult } from '@shared/domain'

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
  | { kind: 'done'; ok: boolean; message: string }

function BatchButton({
  batch,
  menuOpen,
  onOpenMenu
}: {
  batch: Batch
  menuOpen: boolean
  onOpenMenu: (key: string | null) => void
}) {
  const [status, setStatus] = useState<ButtonStatus>({ kind: 'idle' })
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

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
    flash(
      res.accepted,
      res.accepted
        ? `Dispatched ${res.count} marker(s) for ${batch.title}`
        : `Could not dispatch ${batch.title} — daemon offline, batch empty, or a turn is running`
    )
  }

  const empty = batch.count === 0
  const title =
    status.kind === 'busy'
      ? 'Working…'
      : status.kind === 'done'
        ? status.message
        : empty
          ? `${batch.title} — no indianas`
          : `Copy ${batch.title} · ${batch.count} indiana(s); right-click to dispatch`

  return (
    <span className="relative">
      <button
        type="button"
        disabled={empty || status.kind === 'busy'}
        title={title}
        aria-label={title}
        onClick={() => void copy()}
        onContextMenu={(e) => {
          e.preventDefault()
          if (!empty) onOpenMenu(menuOpen ? null : batch.key)
        }}
        className={`flex h-5 min-w-5 items-center justify-center rounded px-1 text-[11px] font-semibold disabled:cursor-not-allowed disabled:opacity-40 ${
          status.kind === 'done' && status.ok ? 'bg-white/25' : 'bg-black/15 hover:bg-black/25'
        }`}
      >
        {batch.label}
      </button>
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
    </span>
  )
}

/**
 * The centered batch cluster: numeric groups 1…n, one button per agent
 * persona (M = mike, L = lisa, …), each graying out while it has no tagged
 * indianas. Click copies the batch (personas carry their own system prompt);
 * right-click offers a one-item dispatch menu.
 */
export function GroupButtons() {
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
          menuOpen={openMenu === batch.key}
          onOpenMenu={setOpenMenu}
        />
      ))}
    </div>
  )
}
