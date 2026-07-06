import { useEffect, useRef } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import {
  TRANSFORMERS,
  $convertFromMarkdownString,
  $convertToMarkdownString,
  type Transformer
} from '@lexical/markdown'

import { createTableTransformer } from './TableMarkdownTransformer'

export type TransformerPack = Transformer[]

const TABLE = createTableTransformer(() => MARKDOWN_TRANSFORMERS)

export const MARKDOWN_TRANSFORMERS: TransformerPack = [TABLE, ...TRANSFORMERS]

interface Props {
  markdown: string
  onChange: (markdown: string) => void
  transformers: TransformerPack
}

/**
 * Bridges Lexical <-> markdown. The composer is remounted per note (keyed by
 * path), so we import once on mount and only emit on change — this keeps the
 * data flow one-directional and avoids import/export feedback loops.
 */
export function MarkdownPlugin({ markdown, onChange, transformers }: Props) {
  const [editor] = useLexicalComposerContext()
  const lastEmitted = useRef<string>(markdown)

  // Import the initial markdown into the editor once it is ready.
  useEffect(() => {
    editor.update(() => {
      $convertFromMarkdownString(markdown, transformers)
      lastEmitted.current = markdown
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Emit markdown on change (debounced via microtask + rAF batching by Lexical).
  useEffect(() => {
    return editor.registerUpdateListener(() => {
      editor.read(() => {
        const md = $convertToMarkdownString(transformers)
        if (md === lastEmitted.current) return
        lastEmitted.current = md
        onChange(md)
      })
    })
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [editor])

  return null
}
