import { describe, expect, it } from 'vitest'
import { ancestorsOf, flattenTree } from './flatten-tree'
import type { TreeNode } from './domain'

const tree: TreeNode = {
  path: '',
  name: 'vault',
  type: 'folder',
  children: [
    {
      path: 'docs',
      name: 'docs',
      type: 'folder',
      children: [
        { path: 'docs/a.md', name: 'a.md', type: 'file' },
        {
          path: 'docs/deep',
          name: 'deep',
          type: 'folder',
          children: [{ path: 'docs/deep/b.md', name: 'b.md', type: 'file' }]
        }
      ]
    },
    { path: 'site', name: 'site', type: 'folder', children: [] },
    { path: 'readme.md', name: 'readme.md', type: 'file' }
  ]
}

describe('flattenTree', () => {
  it('emits the root row first, then flattens depth-first', () => {
    const flat = flattenTree(tree, new Set(), null)
    expect(flat.map((n) => n.path)).toEqual([
      '',
      'docs',
      'docs/a.md',
      'docs/deep',
      'docs/deep/b.md',
      'site',
      'readme.md'
    ])
    expect(flat.map((n) => n.depth)).toEqual([0, 1, 2, 2, 3, 1, 1])
    expect(flat.map((n) => n.index)).toEqual([0, 1, 2, 3, 4, 5, 6])
    expect(flat[0].name).toBe('vault')
    expect(flat[1].parentPath).toBe('')
    expect(flat[4].parentPath).toBe('docs/deep')
  })

  it('skips the whole subtree of a collapsed folder', () => {
    const flat = flattenTree(tree, new Set(['docs']), null)
    expect(flat.map((n) => n.path)).toEqual(['', 'docs', 'site', 'readme.md'])
    expect(flat[1].isExpanded).toBe(false)
  })

  it('collapsing the root folds the whole tree', () => {
    const flat = flattenTree(tree, new Set(['']), null)
    expect(flat.map((n) => n.path)).toEqual([''])
    expect(flat[0].isExpanded).toBe(false)
  })

  it('collapsing a nested folder keeps its siblings visible', () => {
    const flat = flattenTree(tree, new Set(['docs/deep']), null)
    expect(flat.map((n) => n.path)).toEqual([
      '',
      'docs',
      'docs/a.md',
      'docs/deep',
      'site',
      'readme.md'
    ])
  })

  it('marks the active file and empty folders', () => {
    const flat = flattenTree(tree, new Set(), 'docs/a.md')
    expect(flat.find((n) => n.path === 'docs/a.md')?.isActive).toBe(true)
    const site = flat.find((n) => n.path === 'site')
    expect(site?.hasChildren).toBe(false)
    expect(site?.isExpanded).toBe(true)
  })
})

describe('ancestorsOf', () => {
  it('lists every ancestor folder, nearest last', () => {
    expect(ancestorsOf('a/b/c.md')).toEqual(['a', 'a/b'])
    expect(ancestorsOf('top.md')).toEqual([])
  })
})
