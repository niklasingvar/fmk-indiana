import type { AgentJob } from './domain'

/**
 * The live daemon job running a given batch, if any. Batch dispatch job ids
 * are `<batch key>-<marker id>` (`group-2-xy-ab`, `agent-mike-xy-ab` —
 * dispatch.rs `job_prefix`); the trailing `-` keeps `group-2` from claiming
 * a `group-20-…` job. Single-marker auto-run jobs carry bare marker ids and
 * never match.
 */
export function jobForBatch(jobs: AgentJob[], batchKey: string): AgentJob | undefined {
  return jobs.find((job) => job.id.startsWith(`${batchKey}-`))
}
