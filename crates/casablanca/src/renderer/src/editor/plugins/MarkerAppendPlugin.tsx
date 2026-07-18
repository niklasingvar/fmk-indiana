import { useEffect } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { $createParagraphNode, $createTextNode, $getRoot } from 'lexical'

import { onMarkerAppend } from '../marker-events'

/**
 * Appends a `::` marker line to the end of the open document when the tasks
 * panel's composer asks for one. Going through the editor keeps one writer
 * per open note: the ordinary export → autosave path persists the line, so a
 * dirty buffer can never clobber it (the disk-append alternative loses the
 * marker to the next autosave — see useVault's dirty-diverge branch).
 */
export function MarkerAppendPlugin() {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    return onMarkerAppend((commandText) => {
      editor.update(() => {
        const paragraph = $createParagraphNode()
        paragraph.append($createTextNode(commandText))
        $getRoot().append(paragraph)
      })
    })
  }, [editor])

  return null
}
