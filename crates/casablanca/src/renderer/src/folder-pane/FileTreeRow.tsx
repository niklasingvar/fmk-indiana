import { memo } from 'react'
import type { GitFileStatus } from '@shared/domain'
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

const GIT_TINT: Record<GitFileStatus, string> = {
  modified: 'text-git-modified',
  new: 'text-git-new',
  deleted: 'text-git-deleted'
}

/**
 * One tree row, purely presentational (nimbalyst's FileTreeRow split).
 * Rows are divs, not buttons: focus stays on the tree container, which owns
 * the keyboard; `isFocused` draws the roving cursor distinct from active.
 * Git status tints the icon+name; guides mark each open ancestor level.
 */
export const FileTreeRow = memo(function FileTreeRow({
  node,
  isFocused,
  status,
  onClick,
  onContextMenu
}: {
  node: FlatTreeNode
  isFocused: boolean
  status?: GitFileStatus
  onClick: (node: FlatTreeNode) => void
  onContextMenu: (event: React.MouseEvent<HTMLDivElement>) => void
}) {
  const indent = node.depth * INDENT_STEP + INDENT_BASE
  const isFolder = node.type === 'folder'
  const isRoot = node.path === ''
  const tint = status ? GIT_TINT[status] : ''

  return (
    <div
      role="treeitem"
      aria-level={node.depth + 1}
      aria-expanded={isFolder ? node.isExpanded : undefined}
      aria-selected={node.isActive}
      onClick={() => onClick(node)}
      onContextMenu={onContextMenu}
      className={`relative flex h-[26px] w-full cursor-pointer items-center rounded pr-2 text-[13px] transition-colors ${
        node.isActive
          ? 'bg-pane-active ring-1 ring-inset ring-accent'
          : 'hover:bg-pane-hover'
      } ${!node.isActive && isFocused ? 'ring-1 ring-inset ring-accent/50' : ''}`}
      style={{ paddingLeft: indent }}
    >
      {/* Indent guides: one ruler per open ancestor level. */}
      {Array.from({ length: node.depth }, (_, a) => (
        <span
          key={a}
          className="pointer-events-none absolute top-0 bottom-0 w-px bg-pane-border"
          style={{ left: a * INDENT_STEP + INDENT_BASE + 7 }}
        />
      ))}
      <span
        className={`flex min-w-0 items-center gap-1.5 ${
          tint || (isRoot ? 'font-medium text-text-strong' : 'text-text-body')
        }`}
      >
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
            <FileIcon kind={fileKind(node.name)} />
            <span className="truncate">{displayName(node.name)}</span>
          </>
        )}
      </span>
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
      className={`shrink-0 text-text-muted transition-transform ${open ? 'rotate-90' : ''}`}
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

function FileIcon({ kind }: { kind: FileKind }) {
  if (kind === 'html') {
    return (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="shrink-0" aria-hidden>
        <path
          d="M4.5 1.75h4L12 5.25v8a1 1 0 01-1 1H4.5a1 1 0 01-1-1v-10.5a1 1 0 011-1z"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
        <path
          d="M6.4 8.2L5.2 9.4l1.2 1.2M9.6 8.2l1.2 1.2-1.2 1.2"
          stroke="currentColor"
          strokeWidth="1.1"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
    )
  }
  if (kind === 'other') {
    return (
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="shrink-0" aria-hidden>
        <path
          d="M4.5 1.75h4L12 5.25v8a1 1 0 01-1 1H4.5a1 1 0 01-1-1v-10.5a1 1 0 011-1z"
          stroke="currentColor"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
        <path d="M8.25 1.75V5.5H12" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round" />
      </svg>
    )
  }
  // Open book for markdown; sidecars add the two annotation dots.
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" className="shrink-0" aria-hidden>
      <path
        d="M8 3.6c-1.2-.9-3-1.1-5-.9v9.5c2-.2 3.8 0 5 .9 1.2-.9 3-1.1 5-.9V2.7c-2-.2-3.8 0-5 .9z"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinejoin="round"
      />
      <path d="M8 3.6v9.5" stroke="currentColor" strokeWidth="1.2" />
      {kind === 'sidecar' && (
        <>
          <circle cx="5.2" cy="7.4" r="0.8" fill="currentColor" />
          <circle cx="10.8" cy="7.4" r="0.8" fill="currentColor" />
        </>
      )}
    </svg>
  )
}
