import { describe, expect, it } from 'vitest'
import { flattenTree } from '@shared/flatten-tree'
import type { TreeNode } from '@shared/domain'
import { treeKeyAction, typeAheadIndex } from './tree-keys'

const tree: TreeNode = {
  path: '',
  name: 'vault',
  type: 'folder',
  children: [
    {
      path: 'docs',
      name: 'docs',
      type: 'folder',
      children: [{ path: 'docs/a.md', name: 'a.md', type: 'file' }]
    },
    { path: 'site', name: 'site', type: 'folder', children: [] },
    { path: 'zebra.md', name: 'zebra.md', type: 'file' }
  ]
}

const open = flattenTree(tree, new Set(), null)
// open: ['', docs, docs/a.md, site, zebra.md]

describe('treeKeyAction', () => {
  it('focuses the first row when nothing is focused', () => {
    expect(treeKeyAction('ArrowDown', open, null)).toEqual({ kind: 'focus', index: 0 })
  })

  it('moves the cursor and clamps at the edges', () => {
    expect(treeKeyAction('ArrowDown', open, 0)).toEqual({ kind: 'focus', index: 1 })
    expect(treeKeyAction('ArrowDown', open, 4)).toEqual({ kind: 'focus', index: 4 })
    expect(treeKeyAction('ArrowUp', open, 0)).toEqual({ kind: 'focus', index: 0 })
    expect(treeKeyAction('Home', open, 4)).toEqual({ kind: 'focus', index: 0 })
    expect(treeKeyAction('End', open, 0)).toEqual({ kind: 'focus', index: 4 })
  })

  it('right expands a collapsed folder, then steps into it', () => {
    const collapsed = flattenTree(tree, new Set(['docs']), null)
    expect(treeKeyAction('ArrowRight', collapsed, 1)).toEqual({
      kind: 'toggle',
      path: 'docs',
      expand: true
    })
    expect(treeKeyAction('ArrowRight', open, 1)).toEqual({ kind: 'focus', index: 2 })
    expect(treeKeyAction('ArrowRight', open, 3)).toEqual({ kind: 'none' })
  })

  it('left collapses an expanded folder, else jumps to the parent', () => {
    expect(treeKeyAction('ArrowLeft', open, 1)).toEqual({
      kind: 'toggle',
      path: 'docs',
      expand: false
    })
    expect(treeKeyAction('ArrowLeft', open, 2)).toEqual({ kind: 'focus', index: 1 })
    expect(treeKeyAction('ArrowLeft', open, 4)).toEqual({ kind: 'focus', index: 0 })
    expect(treeKeyAction('ArrowLeft', open, 0)).toEqual({ kind: 'toggle', path: '', expand: false })
  })

  it('enter toggles folders and opens files', () => {
    expect(treeKeyAction('Enter', open, 1)).toEqual({ kind: 'toggle', path: 'docs', expand: false })
    expect(treeKeyAction('Enter', open, 4)).toEqual({ kind: 'open', path: 'zebra.md' })
  })
})

describe('typeAheadIndex', () => {
  it('finds the next match forward with wraparound', () => {
    expect(typeAheadIndex(open, null, 'z')).toBe(4)
    expect(typeAheadIndex(open, 4, 'd')).toBe(1)
    expect(typeAheadIndex(open, 1, 's')).toBe(3)
    expect(typeAheadIndex(open, 0, 'nope')).toBeNull()
  })
})
