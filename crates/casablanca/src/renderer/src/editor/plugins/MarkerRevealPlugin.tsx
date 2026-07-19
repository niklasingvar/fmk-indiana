import { useEffect } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { $getRoot, $isElementNode, $isTextNode, type ElementNode } from 'lexical'

import { consumeMarkerReveal, onMarkerReveal } from '../marker-events'
import { revealMatchRank } from './reveal-match'

/** Best-matching block that directly holds text (skips wrapper elements). */
function $findRevealKey(root: ElementNode, rawLine: string): string | null {
  let bestKey: string | null = null
  let bestRank = 0
  const stack: ElementNode[] = [root]
  while (stack.length > 0) {
    const element = stack.pop() as ElementNode
    let holdsText = false
    for (const child of element.getChildren()) {
      if ($isTextNode(child)) holdsText = true
      else if ($isElementNode(child)) stack.push(child)
    }
    if (!holdsText) continue
    const rank = revealMatchRank(element.getTextContent(), rawLine)
    if (rank > bestRank) {
      bestRank = rank
      bestKey = element.getKey()
    }
  }
  return bestKey
}

const RETRY_MS = 100
const MAX_TRIES = 20

/**
 * Scrolls the editor to the line a marker lives on when the markers panel
 * asks for it. The reveal is consumed both on the bus event (note already
 * open) and on mount (note just opened — this plugin mounts with the fresh
 * editor). Content seeding races the mount effect, so the lookup retries
 * briefly instead of assuming the document is already in.
 */
export function MarkerRevealPlugin({ notePath }: { notePath: string }) {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | null = null

    const reveal = (rawLine: string, tries: number): void => {
      const key = editor.getEditorState().read(() => $findRevealKey($getRoot(), rawLine))
      const dom = key ? editor.getElementByKey(key) : null
      if (dom) {
        dom.scrollIntoView({ block: 'center', behavior: 'smooth' })
        return
      }
      if (tries < MAX_TRIES) timer = setTimeout(() => reveal(rawLine, tries + 1), RETRY_MS)
    }

    const take = (): void => {
      const rawLine = consumeMarkerReveal(notePath)
      if (rawLine !== null) reveal(rawLine, 0)
    }

    take()
    const off = onMarkerReveal(take)
    return () => {
      off()
      if (timer) clearTimeout(timer)
    }
  }, [editor, notePath])

  return null
}
