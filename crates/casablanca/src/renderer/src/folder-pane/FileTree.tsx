import { useState } from 'react'
import type { TreeNode } from '@shared/domain'

interface Props {
  node: TreeNode
  depth: number
  activePath: string | null
  onOpen: (rel: string) => void
}

/** Base indent (px) applied to every row, growing by depth. */
const INDENT_STEP = 14
const INDENT_BASE = 8

export function FileTree({ node, depth, activePath, onOpen }: Props) {
  const [open, setOpen] = useState(true)
  const indent = depth * INDENT_STEP + INDENT_BASE

  if (node.type === 'folder') {
    return (
      <div>
        <button
          onClick={() => setOpen((v) => !v)}
          className="group flex h-7 w-full items-center gap-1.5 rounded-md pr-2 text-left text-text-muted transition-colors hover:bg-pane-hover hover:text-gray-200"
          style={{ paddingLeft: indent }}
        >
          <Chevron open={open} />
          <FolderIcon open={open} />
          <span className="truncate text-[13px]">{node.name}</span>
        </button>
        {open &&
          node.children?.map((child) => (
            <FileTree
              key={child.path}
              node={child}
              depth={depth + 1}
              activePath={activePath}
              onOpen={onOpen}
            />
          ))}
      </div>
    )
  }

  const isActive = activePath === node.path
  return (
    <button
      onClick={() => onOpen(node.path)}
      className={`relative flex h-7 w-full items-center gap-1.5 rounded-md pr-2 text-left text-[13px] transition-colors ${
        isActive
          ? 'bg-pane-active text-gray-50'
          : 'text-gray-300 hover:bg-pane-hover hover:text-gray-100'
      }`}
      style={{ paddingLeft: indent }}
    >
      {isActive && (
        <span className="absolute left-0 top-1 bottom-1 w-0.5 rounded-full bg-accent" />
      )}
      {/* Spacer aligns file names under folder names (chevron width). */}
      <span className="w-4 shrink-0" />
      <FileIcon active={isActive} />
      <span className="truncate">{stripMd(node.name)}</span>
    </button>
  )
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

function FileIcon({ active }: { active: boolean }) {
  return (
    <svg
      width="16"
      height="16"
      viewBox="0 0 16 16"
      fill="none"
      className={`shrink-0 ${active ? 'text-accent' : 'text-text-muted'}`}
      aria-hidden
    >
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

function stripMd(name: string): string {
  // Keep the .md visible on annotation sidecars ("page.html.md") so they
  // stay distinguishable from the document they annotate ("page.html").
  if (/\.html?\.md$/i.test(name)) return name
  return name.replace(/\.md$/i, '')
}
