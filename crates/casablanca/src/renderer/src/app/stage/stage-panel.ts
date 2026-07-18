/** Identities for the single shared right-side stage panel. */
export type StagePanelId = 'properties' | 'tasks' | 'history'

export const STAGE_PANELS: readonly {
  id: StagePanelId
  title: string
}[] = [
  { id: 'properties', title: 'Properties' },
  { id: 'tasks', title: 'Tasks' },
  { id: 'history', title: 'History' }
] as const
