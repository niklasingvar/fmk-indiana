import type { AgentJob, ElicitationAction } from '@shared/domain'

import { JobFollowPopover } from './JobFollowPopover'

function jobLabel(job: AgentJob): string {
  if (job.markers.length > 1) return `Group · ${job.markers.length} markers`
  return job.markers[0]?.split('/').pop() ?? 'Agent'
}

/**
 * Compact live-agent indicators for the stage chrome. Every daemon job is a
 * spinner (or awaiting-input mark); click opens the follow popover.
 */
export function AgentIndicators({
  jobs,
  openJobId,
  onOpenJob,
  onAnswer
}: {
  jobs: AgentJob[]
  openJobId: string | null
  onOpenJob: (id: string | null) => void
  onAnswer: (job: AgentJob, action: ElicitationAction, value?: string) => Promise<void>
}) {
  return (
    <div className="flex items-center gap-1">
      {jobs.map((job) => {
        const waiting = job.state === 'awaiting_input'
        const open = openJobId === job.id
        return (
          <span key={job.id} className="relative">
            <button
              onClick={() => onOpenJob(open ? null : job.id)}
              title={waiting ? 'Agent needs an answer' : `${jobLabel(job)} — click to follow`}
              aria-label={waiting ? 'Agent needs an answer' : `Follow ${jobLabel(job)}`}
              className={`flex h-5 w-5 items-center justify-center rounded ${
                waiting ? 'bg-white/25' : 'bg-black/15 hover:bg-black/25'
              }`}
            >
              {waiting ? (
                <span aria-hidden className="text-[11px] font-semibold">
                  ?
                </span>
              ) : (
                <span
                  aria-hidden
                  className="h-3 w-3 shrink-0 animate-spin rounded-full border-2 border-white/40 border-t-white"
                />
              )}
            </button>
            {open && (
              <JobFollowPopover
                job={job}
                onAnswer={(action, value) => onAnswer(job, action, value)}
              />
            )}
          </span>
        )
      })}
    </div>
  )
}
