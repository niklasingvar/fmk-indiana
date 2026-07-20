import { describe, expect, it } from 'vitest'
import type { AgentJob } from './domain'
import { jobForBatch } from './batch-job'

const job = (id: string): AgentJob => ({
  id,
  root: '/vault',
  markers: ['doc.md'],
  state: 'running',
  question: null
})

describe('jobForBatch', () => {
  it('matches a group job by its batch key prefix', () => {
    const jobs = [job('xy-ab'), job('group-2-xy-ab')]
    expect(jobForBatch(jobs, 'group-2')?.id).toBe('group-2-xy-ab')
  })

  it('matches an agent-persona job', () => {
    const jobs = [job('agent-mike-xy-ab')]
    expect(jobForBatch(jobs, 'agent-mike')?.id).toBe('agent-mike-xy-ab')
  })

  it('does not let group-2 claim a group-20 job', () => {
    const jobs = [job('group-20-xy-ab')]
    expect(jobForBatch(jobs, 'group-2')).toBeUndefined()
  })

  it('ignores single-marker auto-run jobs (bare marker ids)', () => {
    const jobs = [job('xy-ab')]
    expect(jobForBatch(jobs, 'group-1')).toBeUndefined()
  })
})
