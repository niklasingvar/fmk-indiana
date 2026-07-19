/** Identities for the single shared right-side stage panel. */
export type StagePanelId = 'properties' | 'markers' | 'tasks' | 'runs' | 'history'

export const STAGE_PANELS: readonly {
  id: StagePanelId
  title: string
}[] = [
  { id: 'properties', title: 'Properties' },
  { id: 'markers', title: 'Indianas' },
  { id: 'tasks', title: 'Tasks' },
  { id: 'runs', title: 'Agent runs' },
  { id: 'history', title: 'History' }
] as const
