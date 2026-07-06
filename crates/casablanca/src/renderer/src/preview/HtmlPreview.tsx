import { useCallback, useEffect, useRef, useState } from 'react'
import { sidecarPath } from '@shared/annotation-line'
import type { AnnotationKind } from '@shared/domain'
import { AnnotationBubble, type Selection } from './AnnotationBubble'

type AnnotatorMessage =
  | ({ type: 'casablanca:select' } & Selection)
  | { type: 'casablanca:invalidate' }

type SaveStatus = { kind: 'idle' } | { kind: 'done'; ok: boolean; message: string }

const SAVE_STATUS_MS = 3000

function toVaultUrl(relPath: string, version: number): string {
  const encoded = relPath.split('/').map(encodeURIComponent).join('/')
  return `vault://local/${encoded}?v=${version}`
}

/**
 * Rendered HTML view with element annotation. The iframe is sandboxed and
 * has no window.api; the injected annotator posts element selections and the
 * bubble here turns them into `::` marker lines via annotation:append.
 */
export function HtmlPreview({ relPath }: { relPath: string }) {
  const iframeRef = useRef<HTMLIFrameElement | null>(null)
  const overlayRef = useRef<HTMLDivElement | null>(null)
  const statusTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const [version, setVersion] = useState(0)
  const [annotating, setAnnotating] = useState(true)
  const [selection, setSelection] = useState<Selection | null>(null)
  const [status, setStatus] = useState<SaveStatus>({ kind: 'idle' })

  // Reload when the document changes on disk (the agent just edited it).
  useEffect(() => {
    return window.api.preview.onChanged((changed) => {
      if (changed === relPath) {
        setSelection(null)
        setVersion((v) => v + 1)
      }
    })
  }, [relPath])

  // Selections and invalidations from the injected annotator.
  useEffect(() => {
    const onMessage = (e: MessageEvent): void => {
      if (e.source !== iframeRef.current?.contentWindow) return
      const msg = e.data as AnnotatorMessage
      if (msg?.type === 'casablanca:select') {
        const { type: _type, ...sel } = msg
        setSelection(sel)
      } else if (msg?.type === 'casablanca:invalidate') {
        setSelection(null)
      }
    }
    window.addEventListener('message', onMessage)
    return () => window.removeEventListener('message', onMessage)
  }, [])

  const sendMode = useCallback((on: boolean) => {
    iframeRef.current?.contentWindow?.postMessage({ type: 'casablanca:set-mode', on }, '*')
  }, [])

  const toggleAnnotating = useCallback(() => {
    setAnnotating((on) => {
      sendMode(!on)
      if (on) setSelection(null)
      return !on
    })
  }, [sendMode])

  useEffect(() => () => {
    if (statusTimer.current) clearTimeout(statusTimer.current)
  }, [])

  const showStatus = useCallback((ok: boolean, message: string) => {
    if (statusTimer.current) clearTimeout(statusTimer.current)
    setStatus({ kind: 'done', ok, message })
    statusTimer.current = setTimeout(() => setStatus({ kind: 'idle' }), SAVE_STATUS_MS)
  }, [])

  const confirm = useCallback(
    async (kind: AnnotationKind, message: string) => {
      if (!selection) return
      try {
        const result = await window.api.annotations.append({
          docRelPath: selection.docRelPath,
          selector: selection.selector,
          excerpt: selection.excerpt,
          kind,
          message
        })
        showStatus(true, `::${kind} → ${result.sidecarRelPath}`)
      } catch (err) {
        showStatus(false, err instanceof Error ? err.message : String(err))
      }
      setSelection(null)
    },
    [selection, showStatus]
  )

  const bounds = {
    width: overlayRef.current?.clientWidth ?? 800,
    height: overlayRef.current?.clientHeight ?? 600
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-pane-border px-6 py-1.5 text-xs text-text-muted">
        <span className="truncate">
          {status.kind === 'done' ? (
            <span className={status.ok ? 'text-accent' : 'text-red-400'} title={status.message}>
              {status.message}
            </span>
          ) : (
            <span>Annotations land in {sidecarPath(relPath)}</span>
          )}
        </span>
        <span className="flex items-center gap-2">
          <button
            onClick={() => setVersion((v) => v + 1)}
            className="rounded border border-pane-border px-2 py-0.5 hover:bg-black/20"
          >
            Reload
          </button>
          <button
            onClick={toggleAnnotating}
            className={`rounded border px-2 py-0.5 hover:bg-black/20 ${
              annotating ? 'border-accent text-accent' : 'border-pane-border'
            }`}
          >
            {annotating ? 'Annotating' : 'Annotate'}
          </button>
        </span>
      </div>
      <div className="relative flex-1">
        <iframe
          ref={iframeRef}
          key={version}
          src={toVaultUrl(relPath, version)}
          sandbox="allow-scripts allow-same-origin allow-forms"
          className="h-full w-full border-0 bg-white"
          onLoad={() => sendMode(annotating)}
        />
        <div ref={overlayRef} className="pointer-events-none absolute inset-0">
          {selection && annotating && (
            <AnnotationBubble
              selection={selection}
              bounds={bounds}
              onConfirm={confirm}
              onClose={() => setSelection(null)}
            />
          )}
        </div>
      </div>
    </div>
  )
}
