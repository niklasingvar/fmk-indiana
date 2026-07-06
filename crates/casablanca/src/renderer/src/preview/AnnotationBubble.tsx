import { useEffect, useState } from 'react'
import { ANNOTATION_KINDS, kindSpec } from '@shared/annotation-line'
import type { AnnotationKind } from '@shared/domain'

/** What the injected annotator reported for the clicked element. */
export interface Selection {
  docRelPath: string
  selector: string
  excerpt: string
  rect: { x: number; y: number; w: number; h: number }
}

const BUBBLE_WIDTH = 288

/**
 * The command bubble, rendered by the host over the preview iframe at the
 * selected element. Reactions confirm on click; other kinds open a message
 * field honoring the marker's contract.
 */
export function AnnotationBubble({
  selection,
  bounds,
  onConfirm,
  onClose
}: {
  selection: Selection
  bounds: { width: number; height: number }
  onConfirm: (kind: AnnotationKind, message: string) => void
  onClose: () => void
}) {
  const [kind, setKind] = useState<AnnotationKind | null>(null)
  const [message, setMessage] = useState('')

  useEffect(() => {
    setKind(null)
    setMessage('')
  }, [selection])

  const left = Math.max(8, Math.min(selection.rect.x, bounds.width - BUBBLE_WIDTH - 8))
  const below = selection.rect.y + selection.rect.h + 8
  const top = Math.max(8, below + 140 > bounds.height ? selection.rect.y - 148 : below)

  const pick = (k: AnnotationKind): void => {
    if (kindSpec(k).message === 'none') {
      onConfirm(k, '')
      return
    }
    setKind(k)
  }

  const spec = kind ? kindSpec(kind) : null
  const canConfirm = spec !== null && (spec.message !== 'required' || message.trim() !== '')

  return (
    <div
      className="pointer-events-auto absolute z-10 rounded-lg border border-pane-border bg-pane p-2 shadow-xl"
      style={{ left, top, width: BUBBLE_WIDTH }}
      onKeyDown={(e) => e.key === 'Escape' && onClose()}
    >
      <div className="mb-2 truncate text-[11px] text-text-muted" title={selection.selector}>
        {selection.excerpt || selection.selector}
      </div>
      <div className="flex flex-wrap gap-1">
        {ANNOTATION_KINDS.map((s) => (
          <button
            key={s.kind}
            onClick={() => pick(s.kind)}
            className={`rounded border px-1.5 py-0.5 text-[11px] hover:bg-pane-hover ${
              kind === s.kind ? 'border-accent text-accent' : 'border-pane-border'
            }`}
          >
            {s.label}
          </button>
        ))}
      </div>
      {spec && (
        <div className="mt-2 flex items-center gap-1">
          <input
            autoFocus
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && canConfirm) onConfirm(kind as AnnotationKind, message)
            }}
            placeholder={spec.message === 'required' ? 'message (required)' : 'message (optional)'}
            className="min-w-0 flex-1 rounded border border-pane-border bg-pane-active px-2 py-1 text-xs outline-none focus:border-accent"
          />
          <button
            disabled={!canConfirm}
            onClick={() => onConfirm(kind as AnnotationKind, message)}
            className="rounded border border-pane-border px-2 py-1 text-xs hover:bg-pane-hover disabled:opacity-50"
          >
            Add
          </button>
        </div>
      )}
    </div>
  )
}
