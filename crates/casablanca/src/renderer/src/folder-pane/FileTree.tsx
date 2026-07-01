import { useState } from 'react'
import type { TreeNode } from '@shared/domain'

interface Props {
  node: TreeNode
  depth: number
  activePath: string | null
  onOpen: (rel: string) => void
}

export function FileTree({ node, depth, activePath, onOpen }: Props) {
  const [open, setOpen] = useState(true)

  if (node.type === 'folder') {
    return (
      <div>
        <button
          onClick={() => setOpen((v) => !v)}
          className="flex w-full items-center gap-1 rounded px-1 py-0.5 text-left hover:bg-pane-hover"
          style={{ paddingLeft: depth * 12 + 4 }}
        >
          <span className="w-3 text-text-muted">{open ? '▾' : '▸'}</span>
          <span className="truncate text-text-muted">{node.name}</span>
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
      className={`flex w-full items-center gap-1 rounded px-1 py-0.5 text-left ${
        isActive ? 'bg-blue-600/20 text-blue-200' : 'hover:bg-pane-hover'
      }`}
      style={{ paddingLeft: depth * 12 + 16 }}
    >
      <span className="truncate">{stripMd(node.name)}</span>
    </button>
  )
}

function stripMd(name: string): string {
  return name.replace(/\.md$/i, '')
}
