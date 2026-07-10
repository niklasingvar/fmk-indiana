import { useEffect, useRef, useState } from 'react'
import type { useVault } from '../storage/useVault'
import { PROJECT_PALETTE } from '@shared/projects'

type Vault = ReturnType<typeof useVault>

/**
 * The folder-pane header control: shows the active project (color dot + name)
 * and opens a menu to switch projects, add a folder, or recolor the active one.
 */
export function ProjectSwitcher({ vault }: { vault: Vault }) {
  const { projects, vaultState, switchProject, addProject, setProjectColor } = vault
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  // Close on outside click.
  useEffect(() => {
    if (!open) return
    const onDown = (e: MouseEvent): void => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', onDown)
    return () => document.removeEventListener('mousedown', onDown)
  }, [open])

  const active = projects.find((p) => p.active)
  const activePath = vaultState.status === 'ready' ? vaultState.rootPath : null

  return (
    <div ref={ref} className="relative min-w-0 flex-1">
      <button
        onClick={() => setOpen((o) => !o)}
        title="Switch project"
        className="flex w-full min-w-0 items-center gap-2 rounded px-1.5 py-1 hover:bg-pane-hover"
      >
        <span className="h-2.5 w-2.5 shrink-0 rounded-full bg-project" />
        <span className="truncate text-sm font-semibold text-text-strong">
          {active?.name ?? 'No project'}
        </span>
        <span className="ml-auto text-text-muted">▾</span>
      </button>

      {open && (
        <div className="absolute left-0 z-20 mt-1 w-60 overflow-hidden rounded-md border border-pane-border bg-pane-active shadow-lg">
          <ul className="max-h-64 overflow-auto py-1">
            {projects.map((p) => (
              <li key={p.rootPath}>
                <button
                  onClick={() => {
                    if (!p.active) void switchProject(p.rootPath)
                    setOpen(false)
                  }}
                  title={p.rootPath}
                  className="flex w-full items-center gap-2 px-3 py-1.5 text-sm hover:bg-pane-hover"
                >
                  <span
                    className="h-2.5 w-2.5 shrink-0 rounded-full"
                    style={{ backgroundColor: `rgb(${p.color})` }}
                  />
                  <span className="flex-1 truncate text-left text-text-body">{p.name}</span>
                  {p.active && <span className="text-text-muted">✓</span>}
                </button>
              </li>
            ))}
          </ul>

          <div className="border-t border-pane-border">
            <button
              onClick={() => {
                void addProject()
                setOpen(false)
              }}
              className="flex w-full items-center gap-2 px-3 py-2 text-sm text-text-muted hover:bg-pane-hover hover:text-text-strong"
            >
              <span className="w-2.5 text-center">＋</span> Open folder…
            </button>
          </div>

          {activePath && (
            <div className="flex flex-wrap gap-1.5 border-t border-pane-border px-3 py-2">
              {PROJECT_PALETTE.map((c) => (
                <button
                  key={c}
                  onClick={() => void setProjectColor(activePath, c)}
                  title="Set project color"
                  aria-label={`Set color rgb(${c})`}
                  className={`h-4 w-4 rounded-full ring-offset-1 ring-offset-pane-active hover:ring-2 hover:ring-pane-border ${
                    active?.color === c ? 'ring-2 ring-text-strong' : ''
                  }`}
                  style={{ backgroundColor: `rgb(${c})` }}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
