import { useCallback, useEffect, useRef, useState } from 'react'
import type { VaultMarker } from '@shared/domain'
import type { useVault } from '../storage/useVault'
import { requestMarkerReveal } from '../editor/marker-events'

type Vault = ReturnType<typeof useVault>

/** Canonical kind order (indiana_core::index::Counts); unknown kinds sink last. */
const KIND_ORDER = [
  'question',
  'hate',
  'love',
  'keep',
  'fix',
  'elaborate',
  'note',
  'action',
  'todo',
  'delete',
  'prompt'
]

const STATUS_DOT: Record<NonNullable<VaultMarker['status']>, string> = {
  working: 'bg-git-modified',
  done: 'bg-git-new',
  failed: 'bg-git-deleted'
}

/** Tree pushes fire per autosave; collapse bursts into one scan. */
const REFRESH_DEBOUNCE_MS = 1000

/** The editor's amber marker accent, shared with the group headers. */
const MARKER_CHIP_STYLE = {
  backgroundColor: 'rgb(var(--marker-bg))',
  color: 'rgb(var(--marker-text))'
} as const

/**
 * A marker's dispatch label in the flag language it was written in:
 * `-mike` for an agent persona, `-1` for a numeric batch, null when loose.
 */
function dispatchLabel(marker: VaultMarker): string | null {
  if (marker.agent) return `-${marker.agent}`
  if (marker.group !== undefined) return `-${marker.group}`
  return null
}

function sortKinds(kinds: string[]): string[] {
  return kinds.sort((a, b) => {
    const ai = KIND_ORDER.indexOf(a)
    const bi = KIND_ORDER.indexOf(b)
    return (ai < 0 ? KIND_ORDER.length : ai) - (bi < 0 ? KIND_ORDER.length : bi)
  })
}

function FilterChip({
  label,
  selected,
  onToggle
}: {
  label: string
  selected: boolean
  onToggle: () => void
}) {
  return (
    <button
      onClick={onToggle}
      aria-pressed={selected}
      className={`rounded-full border px-1.5 py-px font-mono text-[11px] ${
        selected
          ? 'border-transparent font-medium'
          : 'border-pane-border text-text-muted hover:bg-pane-hover'
      }`}
      style={selected ? MARKER_CHIP_STYLE : undefined}
    >
      {label}
    </button>
  )
}

function MarkerRow({ marker, onOpen }: { marker: VaultMarker; onOpen: () => void }) {
  const label = dispatchLabel(marker)
  return (
    <button
      onClick={onOpen}
      title={`${marker.path}:${marker.line}`}
      className="block w-full px-3 py-1 text-left hover:bg-pane-hover"
    >
      <span className="flex items-center gap-1.5">
        {marker.status && (
          <span className={`h-1.5 w-1.5 shrink-0 rounded-full ${STATUS_DOT[marker.status]}`} />
        )}
        <span className="truncate text-xs text-text-strong">
          {marker.message ?? marker.rawToken}
        </span>
      </span>
      <span className="block truncate text-[11px] text-text-muted">
        {label && <span className="font-mono">{label} · </span>}
        {marker.path}:{marker.line}
      </span>
    </button>
  )
}

/**
 * The Indianas overview: every `::` marker in the vault from a read-only
 * scan (core computes, faces render), grouped by kind. Clicking a marker
 * opens its note and asks the editor to scroll to the marker's line via the
 * sticky reveal request — the panel never touches the editor itself.
 *
 * Two single-select filters narrow the list: marker type (kind) and dispatch
 * label (`-1` batches / `-agent` personas). They AND together; clicking a
 * selected chip clears it. A filter whose value vanished from the vault on
 * refresh is ignored rather than stranding an empty panel.
 */
export function MarkersPanel({ vault }: { vault: Vault }) {
  const [available, setAvailable] = useState(true)
  const [markers, setMarkers] = useState<VaultMarker[]>([])
  const [kindFilter, setKindFilter] = useState<string | null>(null)
  const [labelFilter, setLabelFilter] = useState<string | null>(null)
  const { openNote } = vault

  const refresh = useCallback(async (): Promise<void> => {
    const result = await window.api.indiana.markers()
    setAvailable(result.available)
    setMarkers(result.markers)
  }, [])

  const debounce = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => {
    void refresh()
    const off = window.api.tree.onChanged(() => {
      if (debounce.current) clearTimeout(debounce.current)
      debounce.current = setTimeout(() => void refresh(), REFRESH_DEBOUNCE_MS)
    })
    return () => {
      off()
      if (debounce.current) clearTimeout(debounce.current)
    }
  }, [refresh])

  const jump = async (marker: VaultMarker): Promise<void> => {
    // The reveal needs the raw line text; scan lines are 1-based over the file.
    const note = await window.api.notes.read(marker.path)
    const lineText = note.content.split('\n')[marker.line - 1] ?? marker.rawToken
    requestMarkerReveal({ path: marker.path, lineText })
    await openNote(marker.path)
  }

  if (!available) {
    return (
      <div className="p-4 text-xs text-text-muted">
        indiana not found — brew install niklasingvar/fmk-indiana/indiana
      </div>
    )
  }

  const allKinds = sortKinds([...new Set(markers.map((m) => m.kind))])
  const allLabels = [...new Set(markers.map(dispatchLabel).filter((l): l is string => l !== null))]
    .sort()

  // A stale selection (value gone after refresh) is ignored, not applied.
  const activeKind = kindFilter !== null && allKinds.includes(kindFilter) ? kindFilter : null
  const activeLabel = labelFilter !== null && allLabels.includes(labelFilter) ? labelFilter : null

  const visible = markers.filter(
    (m) =>
      (activeKind === null || m.kind === activeKind) &&
      (activeLabel === null || dispatchLabel(m) === activeLabel)
  )
  const visibleKinds = sortKinds([...new Set(visible.map((m) => m.kind))])

  return (
    <div className="flex h-full flex-col overflow-hidden">
      {markers.length > 0 && (
        <div className="shrink-0 space-y-1.5 border-b border-pane-border px-3 py-2">
          <div className="flex flex-wrap items-center gap-1">
            <span className="text-[11px] uppercase tracking-wide text-text-muted">Type</span>
            {allKinds.map((kind) => (
              <FilterChip
                key={kind}
                label={`::${kind}`}
                selected={activeKind === kind}
                onToggle={() => setKindFilter(activeKind === kind ? null : kind)}
              />
            ))}
          </div>
          {allLabels.length > 0 && (
            <div className="flex flex-wrap items-center gap-1">
              <span className="text-[11px] uppercase tracking-wide text-text-muted">Group</span>
              {allLabels.map((label) => (
                <FilterChip
                  key={label}
                  label={label}
                  selected={activeLabel === label}
                  onToggle={() => setLabelFilter(activeLabel === label ? null : label)}
                />
              ))}
            </div>
          )}
        </div>
      )}
      <div className="min-h-0 flex-1 overflow-y-auto">
        {markers.length === 0 ? (
          <div className="p-4 text-xs text-text-muted">No indianas in this vault</div>
        ) : visible.length === 0 ? (
          <div className="p-4 text-xs text-text-muted">No indianas match the filter</div>
        ) : (
          visibleKinds.map((kind) => {
            const group = visible.filter((m) => m.kind === kind)
            return (
              <div key={kind} className="border-b border-pane-border pb-1 last:border-b-0">
                {/* Sticky so the group you are scrolling through stays named. */}
                <div className="sticky top-0 z-10 flex items-center justify-between border-b border-pane-border bg-pane px-3 py-1.5">
                  <span
                    className="rounded px-1.5 py-px font-mono text-[11px] font-medium"
                    style={MARKER_CHIP_STYLE}
                  >
                    ::{kind}
                  </span>
                  <span className="text-[11px] tabular-nums text-text-muted">{group.length}</span>
                </div>
                {group.map((marker) => (
                  <MarkerRow
                    key={`${marker.path}:${marker.line}`}
                    marker={marker}
                    onOpen={() => void jump(marker).catch(() => {})}
                  />
                ))}
              </div>
            )
          })
        )}
      </div>
    </div>
  )
}
