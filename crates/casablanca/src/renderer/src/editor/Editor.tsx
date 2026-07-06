import { useEffect } from 'react'
import { AutoFocusPlugin } from '@lexical/react/LexicalAutoFocusPlugin'
import { LexicalComposer } from '@lexical/react/LexicalComposer'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { LinkPlugin } from '@lexical/react/LexicalLinkPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { TablePlugin } from '@lexical/react/LexicalTablePlugin'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { LinkNode } from '@lexical/link'
import { CodeNode, CodeHighlightNode } from '@lexical/code'
import { TableCellNode, TableNode, TableRowNode } from '@lexical/table'

import { MarkdownPlugin, MARKDOWN_TRANSFORMERS } from './plugins/MarkdownPlugin'

interface Props {
  markdown: string
  onChange: (markdown: string) => void
  /** Called with the raw href when the user cmd/ctrl+clicks a link. */
  onOpenLink?: (href: string) => void
}

/**
 * Cmd/ctrl+click follows links (plain click keeps placing the caret so link
 * text stays editable). The raw href goes to the host, which decides between
 * opening a vault note and the OS browser.
 */
function LinkOpenPlugin({ onOpenLink }: { onOpenLink: (href: string) => void }) {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    const onClick = (e: MouseEvent): void => {
      if (!e.metaKey && !e.ctrlKey) return
      const anchor = (e.target as HTMLElement | null)?.closest?.('a')
      const href = anchor?.getAttribute('href')
      if (!href) return
      e.preventDefault()
      e.stopPropagation()
      onOpenLink(href)
    }
    return editor.registerRootListener((root, prevRoot) => {
      prevRoot?.removeEventListener('click', onClick)
      root?.addEventListener('click', onClick)
    })
  }, [editor, onOpenLink])

  return null
}

export function LexicalEditor({ markdown, onChange, onOpenLink }: Props) {
  const config = {
    namespace: 'casablanca-editor',
    nodes: [
      HeadingNode,
      QuoteNode,
      ListNode,
      ListItemNode,
      LinkNode,
      CodeNode,
      CodeHighlightNode,
      TableNode,
      TableRowNode,
      TableCellNode
    ],
    theme: {
      root: 'casablanca-editor',
      paragraph: 'my-2',
      text: { bold: 'font-semibold', italic: 'italic', code: 'rounded bg-code-bg px-1 font-mono' },
      link: 'text-accent underline',
      list: { ul: 'list-disc pl-6', ol: 'list-decimal pl-6', listitem: 'my-0.5' },
      code: 'block rounded bg-code-bg p-3 font-mono text-sm overflow-auto',
      quote: 'border-l-2 border-pane-border pl-4 italic text-text-muted',
      heading: {
        h1: 'text-2xl font-semibold mt-6 mb-3 text-text-strong',
        h2: 'text-xl font-semibold mt-5 mb-2 text-text-strong',
        h3: 'text-lg font-semibold mt-4 mb-2 text-text-strong'
      },
      table: 'casablanca-table',
      tableCell: 'casablanca-table-cell',
      tableCellHeader: 'casablanca-table-cell-header'
    },
    onError: (error: Error) => {
      console.error('[lexical]', error)
    }
  }

  return (
    <LexicalComposer initialConfig={config}>
      <RichTextPlugin
        contentEditable={<ContentEditable className="casablanca-editor min-h-[60vh] outline-none" />}
        placeholder={<div className="text-text-muted">Start writing…</div>}
        ErrorBoundary={LexicalErrorBoundary}
      />
      <ListPlugin />
      <TablePlugin hasHorizontalScroll />
      <LinkPlugin />
      {onOpenLink && <LinkOpenPlugin onOpenLink={onOpenLink} />}
      <HistoryPlugin />
      <AutoFocusPlugin />
      <MarkdownPlugin markdown={markdown} onChange={onChange} transformers={MARKDOWN_TRANSFORMERS} />
    </LexicalComposer>
  )
}
