import { useCallback, useEffect, useState } from 'react'
import type { CosLogEntry, CosQueue, CosTask } from '@shared/domain'
import type { useVault } from '../storage/useVault'
import { requestMarkerAppend } from '../editor/marker-events'
import { MarkerComposer, type CommandOption } from '../MarkerComposer'

type Vault = ReturnType<typeof useVault>

/** Marker kinds that create tracked tasks (COS_PRD.md queue mapping). */
const TASK_COMMAND_OPTIONS: readonly CommandOption[] = [
  { token: 'todo', label: 'Todo', message: 'required' },
  { token: 'task', label: 'Task', message: 'required' },
  { token: 'action', label: 'Action', message: 'required' }
]

const LOG_TAIL = 15

/** The two files this panel projects; refresh only when one of them changed. */
const TRACKER_PATHS = ['.indiana/chief-of-staff/tasks.md', '.indiana/chief-of-staff/log.md']

const STATE_DOT: Record<CosTask['state'], string> = {
  open: 'bg-white/30',
  working: 'bg-git-modified',
  done: 'bg-git-new',
  failed: 'bg-git-deleted'
}

function TaskRow({ task, onOpen }: { task: CosTask; onOpen?: () => void }) {
  const finished = task.state === 'done' || task.state === 'failed'
  return (
    <button
      onClick={onOpen}
      disabled={!onOpen}
      title={task.origin ? `${task.origin.path}:${task.origin.line}` : task.id}
      className={`block w-full px-3 py-1 text-left ${onOpen ? 'hover:bg-pane-hover' : 'cursor-default'}`}
    >
      <span className="flex items-center gap-1.5">
        <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${STATE_DOT[task.state]}`} />
        <span
          className={`truncate text-xs ${finished ? 'text-text-muted line-through' : 'text-text-strong'}`}
        >
          {task.text}
        </span>
      </span>
      <span className="block truncate pl-3 text-[11px] text-text-muted">
        {task.id}
        {task.origin ? ` · ${task.origin.path}:${task.origin.line}` : ''}
      </span>
    </button>
  )
}

function QueueSection({
  title,
  tasks,
  openOrigin
}: {
  title: string
  tasks: CosTask[]
  openOrigin: (task: CosTask) => (() => void) | undefined
}) {
  return (
    <div className="py-1">
      <div className="px-3 py-1 text-[11px] font-medium uppercase tracking-wide text-text-muted">
        {title}
      </div>
      {tasks.length === 0 ? (
        <div className="px-3 pb-1 text-[11px] text-text-muted">Empty</div>
      ) : (
        tasks.map((task) => <TaskRow key={task.id} task={task} onOpen={openOrigin(task)} />)
      )}
    </div>
  )
}

/**
 * The Chief of Staff panel: both task queues plus the recent action log,
 * read via the Indiana CLI (core computes, faces render — COS_PRD.md) and
 * refreshed by the existing tree push (chokidar already watches `.indiana/`).
 * The composer appends a `::todo`/`::task`/`::action` line to the open note;
 * the daemon's next scan captures it and the panel updates itself.
 */
export function TasksPanel({ vault }: { vault: Vault }) {
  const [available, setAvailable] = useState(true)
  const [tasks, setTasks] = useState<CosTask[]>([])
  const [log, setLog] = useState<CosLogEntry[]>([])
  const [composing, setComposing] = useState(false)
  const { activeNote, openNote } = vault

  const refresh = useCallback(async (): Promise<void> => {
    const [taskResult, logResult] = await Promise.all([
      window.api.cos.tasks(),
      window.api.cos.log(LOG_TAIL)
    ])
    setAvailable(taskResult.available)
    setTasks(taskResult.tasks)
    setLog(logResult.entries)
  }, [])

  // Refresh only when the tracker or log actually changed — not on every
  // tree push (which fires per autosave and would spawn two CLI processes
  // per keystroke burst). The watcher's per-path note event covers both
  // files: they are .md under the watched .indiana/.
  useEffect(() => {
    void refresh()
    return window.api.notes.onChanged((rel) => {
      if (TRACKER_PATHS.includes(rel)) void refresh()
    })
  }, [refresh])

  const byQueue = (queue: CosQueue): CosTask[] => {
    const rows = tasks.filter((task) => task.queue === queue)
    const live = rows.filter((task) => task.state === 'open' || task.state === 'working')
    // Finished tasks sink to the bottom, capped so the queues stay glanceable.
    const finished = rows.filter((task) => task.state === 'done' || task.state === 'failed')
    return [...live, ...finished.slice(-5)]
  }

  // Jump to origin — the v1 "act on a task" gesture (COS_PRD.md).
  const openOrigin = (task: CosTask): (() => void) | undefined => {
    const path = task.origin?.path
    if (!path || !/\.(md|mdx)$/i.test(path)) return undefined
    return () => void openNote(path)
  }

  const canCompose = activeNote !== null && /\.(md|mdx)$/i.test(activeNote.path)

  // Append through the live editor (one writer per open note): the ordinary
  // export → autosave path persists the line, so a dirty buffer can never
  // clobber the marker the way a disk-level append could.
  const submitMarker = (commandText: string): void => {
    if (!activeNote) return
    requestMarkerAppend(commandText)
    setComposing(false)
  }

  if (!available) {
    return (
      <div className="p-4 text-xs text-text-muted">
        indiana not found — brew install niklasingvar/fmk-indiana/indiana
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <QueueSection title="Human" tasks={byQueue('human')} openOrigin={openOrigin} />
        <QueueSection title="Agent" tasks={byQueue('agent')} openOrigin={openOrigin} />
        <div className="border-t border-pane-border py-1">
          <div className="px-3 py-1 text-[11px] font-medium uppercase tracking-wide text-text-muted">
            Recent activity
          </div>
          {log.length === 0 ? (
            <div className="px-3 pb-1 text-[11px] text-text-muted">Nothing has run yet</div>
          ) : (
            log
              .slice()
              .reverse()
              .map((entry, i) => (
                <div
                  key={`${entry.ts}-${entry.id}-${i}`}
                  className="truncate px-3 py-0.5 text-[11px] text-text-muted"
                  title={`${entry.ts} ${entry.event} [${entry.id}] ${entry.detail}`}
                >
                  <span className="text-text-body">{entry.event}</span> {entry.id}
                  {entry.detail ? ` · ${entry.detail}` : ''}
                </div>
              ))
          )}
        </div>
      </div>
      <div className="shrink-0 border-t border-pane-border p-2">
        {composing ? (
          <div>
            <MarkerComposer
              options={TASK_COMMAND_OPTIONS}
              onSubmit={submitMarker}
              onClose={() => setComposing(false)}
            />
            <div className="mt-1 text-[11px] text-text-muted">
              Appends to {activeNote?.name ?? 'the open note'}
            </div>
          </div>
        ) : (
          <button
            disabled={!canCompose}
            title={canCompose ? 'Add a marker to the open note' : 'Open a note to place a marker'}
            onClick={() => setComposing(true)}
            className="w-full rounded border border-pane-border px-2 py-1 text-xs hover:bg-pane-hover disabled:opacity-50"
          >
            Add task
          </button>
        )}
      </div>
    </div>
  )
}
