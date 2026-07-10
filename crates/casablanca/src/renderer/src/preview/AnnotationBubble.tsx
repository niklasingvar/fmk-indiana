import { ANNOTATION_KINDS } from '@shared/annotation-line'
import type { AnnotationKind } from '@shared/domain'
import { MarkerComposer } from '../MarkerComposer'

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
  const left = Math.max(8, Math.min(selection.rect.x, bounds.width - BUBBLE_WIDTH - 8))
  const below = selection.rect.y + selection.rect.h + 8
  const top = Math.max(8, below + 140 > bounds.height ? selection.rect.y - 148 : below)

  const submit = (commandText: string): void => {
    const match = commandText.match(/^::([A-Za-z]+|\?)(?:\s+(.*))?$/)
    if (!match) return
    const spec = ANNOTATION_KINDS.find((option) => option.token === match[1])
    if (!spec) return
    onConfirm(spec.kind, match[2] ?? '')
  }

  return (
    <div
      className="pointer-events-auto absolute z-10 rounded-lg border border-pane-border bg-pane p-2 shadow-xl"
      style={{ left, top, width: BUBBLE_WIDTH }}
      onKeyDown={(e) => e.key === 'Escape' && onClose()}
    >
      <div className="mb-2 truncate text-[11px] text-text-muted" title={selection.selector}>
        {selection.excerpt || selection.selector}
      </div>
      <MarkerComposer
        key={`${selection.docRelPath}:${selection.selector}:${selection.excerpt}`}
        options={ANNOTATION_KINDS}
        onSubmit={submit}
        onClose={onClose}
      />
    </div>
  )
}
