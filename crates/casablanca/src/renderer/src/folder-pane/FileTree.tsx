import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { GitStatusMap, TreeNode } from '@shared/domain'
import { ancestorsOf, flattenTree, type FlatTreeNode } from '@shared/flatten-tree'
import { FileTreeRow } from './FileTreeRow'
import { treeKeyAction, typeAheadIndex } from './tree-keys'

const TYPE_AHEAD_RESET_MS = 500

/**
 * Flat-list file tree (nimbalyst's FlatFileTree pattern): the container owns
 * ONE collapsed-set — persisted per vault so collapse state survives watcher
 * refreshes and restarts — plus the roving keyboard cursor. Rows render from
 * a pure flatten pass; FileTreeRow is presentation only.
 */
export function FileTree({
  tree,
  activePath,
  onOpen,
  vaultKey,
  gitStatus
}: {
  tree: TreeNode
  activePath: string | null
  onOpen: (rel: string) => void
  vaultKey: string
  gitStatus: GitStatusMap
}) {
  const [collapsed, setCollapsed] = useState<Set<string>>(() => loadCollapsed(vaultKey))
  const [focused, setFocused] = useState<number | null>(null)
  const rowRefs = useRef(new Map<number, HTMLDivElement>())
  const typeAhead = useRef({ query: '', timer: null as ReturnType<typeof setTimeout> | null })

  const nodes = useMemo(
    () => flattenTree(tree, collapsed, activePath),
    [tree, collapsed, activePath]
  )

  useEffect(() => saveCollapsed(vaultKey, collapsed), [vaultKey, collapsed])

  // Reveal the active file: expand the root and its ancestors when it changes.
  useEffect(() => {
    if (!activePath) return
    setCollapsed((prev) => {
      const hidden = ['', ...ancestorsOf(activePath)].filter((a) => prev.has(a))
      if (hidden.length === 0) return prev
      const next = new Set(prev)
      for (const a of hidden) next.delete(a)
      return next
    })
  }, [activePath])

  const setExpanded = useCallback((path: string, expand: boolean) => {
    setCollapsed((prev) => {
      const next = new Set(prev)
      if (expand) next.delete(path)
      else next.add(path)
      return next
    })
  }, [])

  const focusRow = useCallback((index: number) => {
    setFocused(index)
    rowRefs.current.get(index)?.scrollIntoView({ block: 'nearest' })
  }, [])

  const clickRow = useCallback(
    (node: FlatTreeNode) => {
      setFocused(node.index)
      if (node.type === 'folder') setExpanded(node.path, !node.isExpanded)
      else onOpen(node.path)
    },
    [onOpen, setExpanded]
  )

  const onKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      // Type-ahead: printable chars find the next matching name.
      if (e.key.length === 1 && !e.metaKey && !e.ctrlKey && !e.altKey) {
        const state = typeAhead.current
        if (state.timer) clearTimeout(state.timer)
        state.query += e.key
        state.timer = setTimeout(() => (state.query = ''), TYPE_AHEAD_RESET_MS)
        const hit = typeAheadIndex(nodes, focused, state.query)
        if (hit !== null) focusRow(hit)
        e.preventDefault()
        return
      }
      const action = treeKeyAction(e.key, nodes, focused)
      if (action.kind === 'none') return
      e.preventDefault()
      if (action.kind === 'focus') focusRow(action.index)
      else if (action.kind === 'toggle') setExpanded(action.path, action.expand)
      else if (action.kind === 'open') onOpen(action.path)
    },
    [nodes, focused, focusRow, onOpen, setExpanded]
  )

  return (
    <div
      role="tree"
      aria-label="Files"
      tabIndex={0}
      onKeyDown={onKeyDown}
      className="outline-none"
    >
      {nodes.map((node) => (
        <div
          key={node.path}
          ref={(el) => {
            if (el) rowRefs.current.set(node.index, el)
            else rowRefs.current.delete(node.index)
          }}
        >
          <FileTreeRow
            node={node}
            isFocused={focused === node.index}
            status={node.path === '' ? undefined : gitStatus[node.path]}
            onClick={clickRow}
          />
        </div>
      ))}
    </div>
  )
}

function storageKey(vaultKey: string): string {
  return `casablanca:collapsed:${vaultKey}`
}

function loadCollapsed(vaultKey: string): Set<string> {
  try {
    const raw = localStorage.getItem(storageKey(vaultKey))
    const parsed: unknown = raw ? JSON.parse(raw) : []
    return new Set(Array.isArray(parsed) ? parsed.filter((p) => typeof p === 'string') : [])
  } catch {
    return new Set()
  }
}

function saveCollapsed(vaultKey: string, collapsed: Set<string>): void {
  try {
    localStorage.setItem(storageKey(vaultKey), JSON.stringify([...collapsed]))
  } catch {
    // Persistence is best-effort; the tree still works session-local.
  }
}
