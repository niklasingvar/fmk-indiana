/**
 * Pure tree flattening for the file tree — the nimbalyst FlatFileTree
 * pattern: the component renders a flat list derived in one pass; expand
 * state lives in ONE set owned by the container, never per row.
 *
 * Casablanca defaults folders to open, so the set stores COLLAPSED paths —
 * a fresh vault renders fully expanded and newly created folders inherit
 * the default without touching persisted state.
 */

import type { TreeNode } from './domain'

export interface FlatTreeNode {
  path: string
  name: string
  type: 'file' | 'folder'
  depth: number
  /** Position in the flattened list — the keyboard cursor moves over this. */
  index: number
  parentPath: string | null
  hasChildren: boolean
  isExpanded: boolean
  isActive: boolean
}

export function flattenTree(
  root: TreeNode,
  collapsed: ReadonlySet<string>,
  activePath: string | null
): FlatTreeNode[] {
  const out: FlatTreeNode[] = []

  const walk = (node: TreeNode, depth: number, parentPath: string | null): void => {
    for (const child of node.children ?? []) {
      const isFolder = child.type === 'folder'
      const isExpanded = isFolder && !collapsed.has(child.path)
      out.push({
        path: child.path,
        name: child.name,
        type: child.type,
        depth,
        index: out.length,
        parentPath,
        hasChildren: isFolder && (child.children?.length ?? 0) > 0,
        isExpanded,
        isActive: child.path === activePath
      })
      if (isExpanded) walk(child, depth + 1, child.path)
    }
  }

  walk(root, 0, null)
  return out
}

/** Every ancestor folder path of a vault-relative path (nearest last). */
export function ancestorsOf(path: string): string[] {
  const parts = path.split('/')
  const out: string[] = []
  for (let i = 1; i < parts.length; i++) out.push(parts.slice(0, i).join('/'))
  return out
}
