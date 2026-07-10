/**
 * Pure keyboard logic for the file tree (nimbalyst's reducer pattern):
 * a key + the flat list + the cursor in, one action out. The component
 * applies actions; nothing here touches state or the DOM.
 */

import type { FlatTreeNode } from '@shared/flatten-tree'

export type TreeAction =
  | { kind: 'focus'; index: number }
  | { kind: 'open'; path: string }
  | { kind: 'toggle'; path: string; expand: boolean }
  | { kind: 'delete'; path: string }
  | { kind: 'none' }

const NAV_KEYS = new Set(['ArrowDown', 'ArrowUp', 'ArrowRight', 'ArrowLeft', 'Enter', 'Home', 'End'])
const DELETE_KEYS = new Set(['Backspace', 'Delete'])

export function treeKeyAction(
  key: string,
  nodes: readonly FlatTreeNode[],
  focusedIndex: number | null
): TreeAction {
  if (nodes.length === 0) return { kind: 'none' }
  if (focusedIndex === null || focusedIndex < 0 || focusedIndex >= nodes.length) {
    if (DELETE_KEYS.has(key)) return { kind: 'none' }
    if (!NAV_KEYS.has(key)) return { kind: 'none' }
    return { kind: 'focus', index: 0 }
  }
  const node = nodes[focusedIndex]

  if (DELETE_KEYS.has(key)) {
    return node.path === '' ? { kind: 'none' } : { kind: 'delete', path: node.path }
  }
  if (!NAV_KEYS.has(key)) return { kind: 'none' }

  switch (key) {
    case 'ArrowDown':
      return { kind: 'focus', index: Math.min(focusedIndex + 1, nodes.length - 1) }
    case 'ArrowUp':
      return { kind: 'focus', index: Math.max(focusedIndex - 1, 0) }
    case 'ArrowRight':
      if (node.type === 'folder' && !node.isExpanded) {
        return { kind: 'toggle', path: node.path, expand: true }
      }
      if (node.type === 'folder' && node.hasChildren) {
        return { kind: 'focus', index: focusedIndex + 1 }
      }
      return { kind: 'none' }
    case 'ArrowLeft': {
      if (node.type === 'folder' && node.isExpanded && node.hasChildren) {
        return { kind: 'toggle', path: node.path, expand: false }
      }
      if (node.parentPath === null) return { kind: 'none' }
      const parent = nodes.findIndex((n) => n.path === node.parentPath)
      return parent === -1 ? { kind: 'none' } : { kind: 'focus', index: parent }
    }
    case 'Enter':
      return node.type === 'folder'
        ? { kind: 'toggle', path: node.path, expand: !node.isExpanded }
        : { kind: 'open', path: node.path }
    case 'Home':
      return { kind: 'focus', index: 0 }
    case 'End':
      return { kind: 'focus', index: nodes.length - 1 }
    default:
      return { kind: 'none' }
  }
}

/** Type-ahead: first name starting with the query, searching forward with wraparound. */
export function typeAheadIndex(
  nodes: readonly FlatTreeNode[],
  fromIndex: number | null,
  query: string
): number | null {
  if (nodes.length === 0 || query === '') return null
  const q = query.toLowerCase()
  const start = fromIndex === null ? 0 : fromIndex + 1
  for (let step = 0; step < nodes.length; step++) {
    const i = (start + step) % nodes.length
    if (nodes[i].name.toLowerCase().startsWith(q)) return i
  }
  return null
}
