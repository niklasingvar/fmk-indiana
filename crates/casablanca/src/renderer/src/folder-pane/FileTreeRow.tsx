import { memo } from 'react'
import type { FlatTreeNode } from '@shared/flatten-tree'

/** Base indent (px) applied to every row, growing by depth. */
const INDENT_STEP = 16
const INDENT_BASE = 8

export type FileKind = 'markdown' | 'html' | 'sidecar' | 'other'

export function fileKind(name: string): FileKind {
  if (/\.html?\.md$/i.test(name)) return 'sidecar'
  if (/\.html?$/i.test(name)) return 'html'
  if (/\.mdx?$/i.test(name)) return 'markdown'
  return 'other'
}

/**
 * One tree row, purely presentational (nimbalyst's FileTreeRow split).
 * Rows are divs, not buttons: focus stays on the tree container, which owns
 * the keyboard; `isFocused` draws the roving cursor distinct from active.
 */
export const FileTreeRow = memo(function FileTreeRow({
  node,
  isFocused,
  onClick
}: {
  node: FlatTreeNode
  isFocused: boolean
  onClick: (node: FlatTreeNode) => void
}) {
  const indent = node.depth * INDENT_STEP + INDENT_BASE
  const isFolder = node.type === 'folder'

  return (
    <div
      role="treeitem"
      aria-level={node.depth + 1}
      aria-expanded={isFolder ? node.isExpanded : undefined}
      aria-selected={node.isActive}
      onClick={() => onClick(node)}
      className={`relative flex h-7 w-full cursor-pointer items-center gap-1.5 rounded-md pr-2 text-[13px] transition-colors ${
        node.isActive
          ? 'bg-pane-active text-gray-50'
          : isFolder
            ? 'text-text-muted hover:bg-pane-hover hover:text-gray-200'
            : 'text-gray-300 hover:bg-pane-hover hover:text-gray-100'
      } ${isFocused ? 'ring-1 ring-inset ring-accent/60' : ''}`}
      style={{ paddingLeft: indent }}
    >
      {node.isActive && (
        <span className="absolute left-0 top-1 bottom-1 w-0.5 rounded-full bg-accent" />
      )}
      {isFolder ? (
        <>
          <Chevron open={node.isExpanded} />
          <FolderIcon open={node.isExpanded} />
          <span className="truncate">{node.name}</span>
        </>
      ) : (
        <>
          {/* Spacer aligns file names under folder names (chevron width). */}
          <span className="w-4 shrink-0" />
          <FileIcon kind={fileKind(node.name)} active={node.isActive} />
          <span className="truncate">{displayName(node.name)}</span>
        </>
      )}
    </div>
  )
})

function displayName(name: string): string {
  // Keep the .md visible on annotation sidecars ("page.html.md") so they
  // stay distinguishable from the document they annotate ("page.html").
  if (/\.html?\.md$/i.test(name)) return name
  return name.replace(/\.md$/i, '')
}

function Chevron({ open }: { open: boolean }) {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      className={`shrink-0 transition-transform ${open ? 'rotate-90' : ''}`}
      aria-hidden
    >
      <path
        d="M6 4l4 4-4 4"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function FolderIcon({ open }: { open: boolean }) {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="shrink-0" aria-hidden>
      {open ? (
        <path
          d="M2 5.5V4a1 1 0 011-1h2.6a1 1 0 01.7.3l.9.9H12a1 1 0 011 1v.8M2 5.5h11.4a.6.6 0 01.58.75l-1 4A1 1 0 0112 12H3a1 1 0 01-1-1V5.5z"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
      ) : (
        <path
          d="M2 4.5a1 1 0 011-1h2.6a1 1 0 01.7.3l.9.9H13a1 1 0 011 1V11a1 1 0 01-1 1H3a1 1 0 01-1-1V4.5z"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
      )}
    </svg>
  )
}

function FileIcon({ kind, active }: { kind: FileKind; active: boolean }) {
  const tone = active ? 'text-accent' : kind === 'sidecar' ? 'text-accent/70' : 'text-text-muted'
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      className={`shrink-0 ${tone}`}
      aria-hidden
    >
      <path
        d="M4.5 1.75h4L12 5.25v8a1 1 0 01-1 1H4.5a1 1 0 01-1-1v-10.5a1 1 0 011-1z"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinejoin="round"
      />
      <path d="M8.25 1.75V5.5H12" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round" />
      {kind === 'html' && (
        <path
          d="M6.4 8.2L5.2 9.4l1.2 1.2M9.6 8.2l1.2 1.2-1.2 1.2"
          stroke="currentColor"
          strokeWidth="1.1"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      )}
      {kind === 'sidecar' && (
        <>
          <circle cx="6.4" cy="9.5" r="0.9" fill="currentColor" />
          <circle cx="9.6" cy="9.5" r="0.9" fill="currentColor" />
        </>
      )}
    </svg>
  )
}
