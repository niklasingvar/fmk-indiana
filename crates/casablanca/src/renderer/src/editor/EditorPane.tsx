import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { isHtmlPath } from '@shared/annotation-line'
import { isExternalLink, resolveVaultLink } from '@shared/resolve-link'
import type { TreeNode } from '@shared/domain'
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
  const { activeNote, draft, setDraftBody, saving, openNote, goBack, goForward, canBack, canForward, tree } =
    vault
  const isHtml = activeNote !== null && isHtmlPath(activeNote.path)

  // Every file in the vault, for the @-mention suggestion list.
  const filePaths = useMemo(() => {
    const out: string[] = []
    const walk = (n: TreeNode): void => {
      for (const c of n.children ?? []) {
        if (c.type === 'file') out.push(c.path)
        else walk(c)
      }
    }
    if (tree) walk(tree)
    return out
  }, [tree])

  // Cmd/ctrl+click on a link: vault-internal targets open in the editor,
  // external ones go through main's window-open handler to the OS browser.
  const openLink = useCallback(
    (href: string) => {
      if (isExternalLink(href)) {
        window.open(href)
        return
      }
      if (!activeNote) return
      const target = resolveVaultLink(activeNote.path, href)
      if (target) void openNote(target).catch(() => {})
    },
    [activeNote, openNote]
  )

  // Browser-style shortcuts: Cmd/Ctrl+[ back, Cmd/Ctrl+] forward.
  useEffect(() => {
    const onKey = (e: KeyboardEvent): void => {
      if (!e.metaKey && !e.ctrlKey) return
      if (e.key === '[') {
        e.preventDefault()
        void goBack()
      } else if (e.key === ']') {
        e.preventDefault()
        void goForward()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [goBack, goForward])

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center justify-between border-b border-pane-border px-6 py-2 text-xs text-text-muted">
        <span className="flex min-w-0 items-center gap-2">
          <button
            onClick={() => void goBack()}
            disabled={!canBack}
            title="Back (⌘[)"
            className="rounded border border-pane-border px-1.5 py-0.5 hover:bg-pane-hover disabled:opacity-40"
          >
            ‹
          </button>
          <button
            onClick={() => void goForward()}
            disabled={!canForward}
            title="Forward (⌘])"
            className="rounded border border-pane-border px-1.5 py-0.5 hover:bg-pane-hover disabled:opacity-40"
          >
            ›
          </button>
          <span className="truncate font-mono">{activeNote ? activeNote.path : 'No note open'}</span>
        </span>
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
            <LexicalEditor
              key={activeNote.path}
              markdown={draft.body}
              onChange={setDraftBody}
              onOpenLink={openLink}
              notePath={activeNote.path}
              filePaths={filePaths}
            />
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
