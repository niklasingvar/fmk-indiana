import { AutoFocusPlugin } from '@lexical/react/LexicalAutoFocusPlugin'
import { LexicalComposer } from '@lexical/react/LexicalComposer'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { LinkPlugin } from '@lexical/react/LexicalLinkPlugin'
import { ListPlugin } from '@lexical/react/LexicalListPlugin'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { LinkNode } from '@lexical/link'
import { CodeNode, CodeHighlightNode } from '@lexical/code'

import { MarkdownPlugin, MARKDOWN_TRANSFORMERS } from './plugins/MarkdownPlugin'

interface Props {
  markdown: string
  onChange: (markdown: string) => void
}

export function LexicalEditor({ markdown, onChange }: Props) {
  const config = {
    namespace: 'casablanca-editor',
    nodes: [
      HeadingNode,
      QuoteNode,
      ListNode,
      ListItemNode,
      LinkNode,
      CodeNode,
      CodeHighlightNode
    ],
    theme: {
      root: 'casablanca-editor',
      paragraph: 'my-2',
      text: { bold: 'font-semibold', italic: 'italic', code: 'rounded bg-black/30 px-1 font-mono' },
      link: 'text-blue-400 underline',
      list: { ul: 'list-disc pl-6', ol: 'list-decimal pl-6', listitem: 'my-0.5' },
      code: 'block rounded bg-black/40 p-3 font-mono text-sm overflow-auto',
      quote: 'border-l-2 border-pane-border pl-4 italic text-text-muted',
      heading: {
        h1: 'text-2xl font-semibold mt-6 mb-3',
        h2: 'text-xl font-semibold mt-5 mb-2',
        h3: 'text-lg font-semibold mt-4 mb-2'
      }
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
      <LinkPlugin />
      <HistoryPlugin />
      <AutoFocusPlugin />
      <MarkdownPlugin markdown={markdown} onChange={onChange} transformers={MARKDOWN_TRANSFORMERS} />
    </LexicalComposer>
  )
}
