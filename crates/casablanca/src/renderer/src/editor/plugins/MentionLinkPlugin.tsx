import { useCallback, useMemo, useState } from 'react'
import { createPortal } from 'react-dom'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import {
  LexicalTypeaheadMenuPlugin,
  MenuOption,
  useBasicTypeaheadTriggerMatch
} from '@lexical/react/LexicalTypeaheadMenuPlugin'
import { $createTextNode, type TextNode } from 'lexical'
import { $createLinkNode } from '@lexical/link'
import { relativeLink } from '@shared/resolve-link'

const MAX_SUGGESTIONS = 8

class FileOption extends MenuOption {
  path: string
  constructor(path: string) {
    super(path)
    this.path = path
  }
}

function labelFor(path: string): string {
  const name = path.split('/').pop() ?? path
  return name.replace(/\.md$/i, '')
}

/**
 * Typing `@` suggests vault files; picking one inserts a markdown link whose
 * href is relative to this note (so it works in the editor AND on GitHub).
 */
export function MentionLinkPlugin({
  notePath,
  filePaths
}: {
  notePath: string
  filePaths: string[]
}) {
  const [editor] = useLexicalComposerContext()
  const [query, setQuery] = useState<string | null>(null)
  const triggerFn = useBasicTypeaheadTriggerMatch('@', { minLength: 0 })

  const options = useMemo(() => {
    const q = (query ?? '').toLowerCase()
    return filePaths
      .filter((p) => p !== notePath && p.toLowerCase().includes(q))
      .slice(0, MAX_SUGGESTIONS)
      .map((p) => new FileOption(p))
  }, [filePaths, notePath, query])

  const onSelect = useCallback(
    (option: FileOption, nodeToReplace: TextNode | null, closeMenu: () => void) => {
      editor.update(() => {
        const link = $createLinkNode(relativeLink(notePath, option.path))
        link.append($createTextNode(labelFor(option.path)))
        if (nodeToReplace) nodeToReplace.replace(link)
        const space = $createTextNode(' ')
        link.insertAfter(space)
        space.select()
        closeMenu()
      })
    },
    [editor, notePath]
  )

  return (
    <LexicalTypeaheadMenuPlugin<FileOption>
      onQueryChange={setQuery}
      onSelectOption={onSelect}
      triggerFn={triggerFn}
      options={options}
      menuRenderFn={(anchorRef, { selectedIndex, selectOptionAndCleanUp, setHighlightedIndex }) =>
        anchorRef.current && options.length > 0
          ? createPortal(
              <ul className="z-50 mt-1 w-72 overflow-hidden rounded-md border border-pane-border bg-pane py-1 text-[13px] shadow-xl">
                {options.map((option, i) => (
                  <li
                    key={option.key}
                    role="option"
                    aria-selected={selectedIndex === i}
                    onMouseEnter={() => setHighlightedIndex(i)}
                    onMouseDown={(e) => {
                      e.preventDefault()
                      selectOptionAndCleanUp(option)
                    }}
                    className={`cursor-pointer truncate px-2 py-1 ${
                      selectedIndex === i ? 'bg-pane-hover text-text-strong' : 'text-text-body'
                    }`}
                  >
                    {option.path}
                  </li>
                ))}
              </ul>,
              anchorRef.current
            )
          : null
      }
    />
  )
}
