import { useCallback, useEffect, useRef, useState } from 'react'
import type { AgentJob, AgentJobsResult, ElicitationAction } from '@shared/domain'

import { onOpenJobFollow } from './job-events'

const POLL_MS = 1_000

/**
 * Polls the daemon's live ACP job projection. Casablanca is a face: it never
 * owns job lifecycle. Offline reads as empty jobs, not an error.
 */
export function useAgentJobs(): {
  online: boolean
  jobs: AgentJob[]
  openJobId: string | null
  setOpenJobId: (id: string | null) => void
  answer: (job: AgentJob, action: ElicitationAction, value?: string) => Promise<void>
} {
  const [state, setState] = useState<AgentJobsResult>({ online: false, jobs: [] })
  const [openJobId, setOpenJobId] = useState<string | null>(null)
  const stateRef = useRef(state)
  stateRef.current = state

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

  // The editor's inline working spinner opens the follow view by marker id;
  // group jobs carry the id as a `group-N-<id>` suffix.
  useEffect(() => {
    return onOpenJobFollow((markerId) => {
      const job = stateRef.current.jobs.find(
        (candidate) => candidate.id === markerId || candidate.id.endsWith(`-${markerId}`)
      )
      if (job) setOpenJobId(job.id)
    })
  }, [])

  const answer = useCallback(
    async (job: AgentJob, action: ElicitationAction, value?: string): Promise<void> => {
      const result = await window.api.indiana.answerJob(job.id, action, value)
      if (!result.accepted) throw new Error('This agent question is no longer waiting')
      await refresh()
    },
    [refresh]
  )

  return {
    online: state.online,
    jobs: state.jobs,
    openJobId,
    setOpenJobId,
    answer
  }
}
