/**
 * Repairs markdown links whose text is inline code: `[` + `` `file.md` `` +
 * `](./file.md)`. Lexical's importer applies code-format transformers before
 * text-match ones, so the code span splits the text node and the stock LINK
 * matcher never sees the full pattern — the link imports as plain text.
 *
 * This pass runs once after import and merges the three-node shape
 * (text ending "[", code-formatted text, text starting "](url)") into a real
 * LinkNode with a code-formatted child. The stock LINK export serializes it
 * back to the exact source text, so the round-trip stays byte-stable.
 */

import {
  $createTextNode,
  $getRoot,
  $isElementNode,
  $isTextNode,
  type ElementNode,
  type TextNode
} from 'lexical'
import { $createLinkNode, $isLinkNode, LinkNode } from '@lexical/link'
import type { TextMatchTransformer } from '@lexical/markdown'

export function $mergeCodeLinks(): void {
  visit($getRoot())
}

/**
 * Export-side companion: the stock LINK transformer serializes a link whose
 * only child is code-formatted as `` `[text](url)` `` (format hoisted around
 * the whole link). Ordered before it, this emits the source form
 * `` [`text`](url) `` so the merge pass round-trips byte-stable. Import stays
 * with $mergeCodeLinks (the regexes never match).
 */
export const CODE_LINK_EXPORT: TextMatchTransformer = {
  dependencies: [LinkNode],
  export: (node) => {
    if (!$isLinkNode(node) || node.getChildrenSize() !== 1) return null
    const child = node.getFirstChild()
    if (!$isTextNode(child) || !child.hasFormat('code')) return null
    return `[\`${child.getTextContent()}\`](${node.getURL()})`
  },
  importRegExp: /$^/,
  regExp: /$^/,
  replace: () => {},
  trigger: '',
  type: 'text-match'
}

function visit(element: ElementNode): void {
  let child = element.getFirstChild()
  while (child) {
    if ($isElementNode(child) && !$isLinkNode(child)) {
      visit(child)
      child = child.getNextSibling()
      continue
    }
    if ($isTextNode(child)) {
      const link = tryMergeAt(child)
      if (link) {
        child = link.getNextSibling()
        continue
      }
    }
    child = child.getNextSibling()
  }
}

function tryMergeAt(open: TextNode): LinkNode | null {
  if (open.getFormat() !== 0 || !open.getTextContent().endsWith('[')) return null
  const code = open.getNextSibling()
  if (!$isTextNode(code) || !code.hasFormat('code')) return null
  const close = code.getNextSibling()
  if (!$isTextNode(close) || close.getFormat() !== 0) return null
  const match = close.getTextContent().match(/^\]\(([^()\s]+)\)/)
  if (!match) return null

  const link = $createLinkNode(match[1])
  const inner = $createTextNode(code.getTextContent())
  inner.setFormat(code.getFormat())
  link.append(inner)
  close.insertBefore(link)
  code.remove()

  const openText = open.getTextContent().slice(0, -1)
  if (openText === '') open.remove()
  else open.setTextContent(openText)

  const closeRest = close.getTextContent().slice(match[0].length)
  if (closeRest === '') close.remove()
  else close.setTextContent(closeRest)

  return link
}
