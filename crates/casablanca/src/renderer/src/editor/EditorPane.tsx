import { useCallback, useEffect, useRef, useState } from 'react'
import { isHtmlPath } from '@shared/annotation-line'
import type { useVault } from '../storage/useVault'
import { initTheme, setTheme, type Theme } from '../app/theme'
import { HtmlPreview } from '../preview/HtmlPreview'
import { LexicalEditor } from './Editor'

type Vault = ReturnType<typeof useVault>

type CopyStatus = { kind: 'idle' } | { kind: 'busy' } | { kind: 'done'; ok: boolean; message: string }

const COPY_STATUS_MS = 4000

/**
 * Vault-wide `Copy all`: asks main to run `indiana copy` for the vault root
 * and surfaces the result inline. The editor never compiles anything itself.
 */
function CopyAllButton() {
  const [status, setStatus] = useState<CopyStatus>({ kind: 'idle' })
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => () => {
    if (timer.current) clearTimeout(timer.current)
  }, [])

  const copyAll = useCallback(async () => {
    if (timer.current) clearTimeout(timer.current)
    setStatus({ kind: 'busy' })
    const res = await window.api.indiana.copyAll().catch((err: unknown) => ({
      ok: false,
      message: err instanceof Error ? err.message : String(err)
    }))
    setStatus({ kind: 'done', ...res })
    timer.current = setTimeout(() => setStatus({ kind: 'idle' }), COPY_STATUS_MS)
  }, [])

  return (
    <span className="flex items-center gap-2">
      {status.kind === 'done' && (
        <span
          className={`max-w-md truncate ${status.ok ? 'text-text-muted' : 'text-git-deleted'}`}
          title={status.message}
        >
          {status.message}
        </span>
      )}
      <button
        onClick={copyAll}
        disabled={status.kind === 'busy'}
        className="rounded border border-pane-border px-2 py-0.5 text-xs hover:bg-pane-hover disabled:opacity-50"
      >
        {status.kind === 'busy' ? 'Copying…' : 'Copy all'}
      </button>
    </span>
  )
}

function ThemeToggle() {
  const [theme, setThemeState] = useState<Theme>(() => initTheme())
  const flip = (): void => {
    const next: Theme = theme === 'light' ? 'dark' : 'light'
    setTheme(next)
    setThemeState(next)
  }
  return (
    <button
      onClick={flip}
      title={theme === 'light' ? 'Switch to dark theme' : 'Switch to light theme'}
      className="rounded border border-pane-border px-2 py-0.5 text-xs hover:bg-pane-hover"
    >
      {theme === 'light' ? '☾' : '☀'}
    </button>
  )
}

export function EditorPane({ vault }: { vault: Vault }) {
  const { activeNote, draft, setDraftBody, saving } = vault
  const isHtml = activeNote !== null && isHtmlPath(activeNote.path)

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-pane-border px-6 py-2 text-xs text-text-muted">
        <span className="truncate font-mono">{activeNote ? activeNote.path : 'No note open'}</span>
        <span className="flex items-center gap-3">
          {activeNote && !isHtml && <span>{saving ? 'Saving…' : 'Saved'}</span>}
          <CopyAllButton />
          <ThemeToggle />
        </span>
      </header>
      {activeNote && isHtml ? (
        <div className="flex-1 overflow-hidden">
          <HtmlPreview key={activeNote.path} relPath={activeNote.path} />
        </div>
      ) : activeNote && draft ? (
        <div className="flex-1 overflow-auto">
          <div className="mx-auto max-w-3xl px-8 py-8">
            <LexicalEditor key={activeNote.path} markdown={draft.body} onChange={setDraftBody} />
          </div>
        </div>
      ) : (
        <div className="flex flex-1 items-center justify-center text-text-muted">
          Select a note from the left, or create a new one.
        </div>
      )}
    </div>
  )
}
