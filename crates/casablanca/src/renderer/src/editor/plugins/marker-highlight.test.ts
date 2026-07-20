// ::ignore
import { describe, expect, it } from 'vitest'
import { createHeadlessEditor } from '@lexical/headless'
import { $convertToMarkdownString } from '@lexical/markdown'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { LinkNode } from '@lexical/link'
import { CodeNode, CodeHighlightNode } from '@lexical/code'
import { TableCellNode, TableNode, TableRowNode } from '@lexical/table'
import { $getRoot, $isElementNode, $isTextNode, TextNode, type LexicalNode } from 'lexical'
import { $importMarkdown, MARKDOWN_TRANSFORMERS } from './MarkdownPlugin'
import {
  highlightMarker,
  INDIANA_MARKER_STYLE,
  stripMarkerStylesFromClipboard
} from './MarkerHighlightPlugin'

function makeEditor() {
  return createHeadlessEditor({
    namespace: 'casablanca-marker-highlight-test',
    nodes: [HeadingNode, QuoteNode, ListNode, ListItemNode, LinkNode, CodeNode, CodeHighlightNode, TableNode, TableRowNode, TableCellNode],
    onError: (error: Error) => {
      throw error
    }
  })
}

function styleMarkers(node: LexicalNode): void {
  if ($isTextNode(node)) {
    highlightMarker(node)
    return
  }
  if ($isElementNode(node)) node.getChildren().forEach(styleMarkers)
}

function readText(editor: ReturnType<typeof makeEditor>): Array<{ text: string; style: string; code: boolean }> {
  const nodes: Array<{ text: string; style: string; code: boolean }> = []
  editor.read(() => {
    const visit = (node: LexicalNode): void => {
      if ($isTextNode(node)) {
        nodes.push({ text: node.getTextContent(), style: node.getStyle(), code: node.hasFormat('code') })
        return
      }
      if ($isElementNode(node)) node.getChildren().forEach(visit)
    }
    visit($getRoot())
  })
  return nodes
}

describe('marker highlighting', () => {
  it('styles the marker suffix without changing markdown', () => {
    const markdown = 'A paragraph ::fix tighten this'
    const editor = makeEditor()

    editor.update(() => {
      $importMarkdown(markdown, MARKDOWN_TRANSFORMERS)
      styleMarkers($getRoot())
    }, { discrete: true })

    const nodes = readText(editor)
    expect(nodes).toContainEqual({ text: 'A paragraph ', style: '', code: false })
    expect(nodes).toContainEqual({ text: '::fix tighten this', style: INDIANA_MARKER_STYLE, code: false })

    let exported = ''
    editor.read(() => {
      exported = $convertToMarkdownString(MARKDOWN_TRANSFORMERS)
    })
    expect(exported).toBe(markdown)
  })

  it('keeps the full command highlighted across split text nodes', () => {
    const markdown = 'A paragraph ::fix lets go!!!'
    const editor = makeEditor()

    editor.update(() => {
      $importMarkdown(markdown, MARKDOWN_TRANSFORMERS)
      const paragraph = $getRoot().getFirstChild()
      if (!$isElementNode(paragraph)) throw new Error('expected paragraph')
      const text = paragraph.getFirstChild()
      if (!$isTextNode(text)) throw new Error('expected text')
      const markerOffset = text.getTextContent().indexOf('::')
      const [, markerAndMessage] = text.splitText(markerOffset)
      markerAndMessage.splitText(3)
      styleMarkers($getRoot())
    }, { discrete: true })

    expect(readText(editor)).toEqual([
      { text: 'A paragraph ', style: '', code: false },
      { text: '::fix lets go!!!', style: INDIANA_MARKER_STYLE, code: false }
    ])
  })

  it('keeps editing and copied text stable after a marker changes', () => {
    const editor = makeEditor()
    editor.registerNodeTransform(TextNode, highlightMarker)

    editor.update(() => {
      $importMarkdown('A paragraph ::fix tighten this', MARKDOWN_TRANSFORMERS)
    }, { discrete: true })

    editor.update(() => {
      const paragraph = $getRoot().getFirstChild()
      if (!$isElementNode(paragraph)) throw new Error('expected paragraph')
      const marker = paragraph.getLastChild()
      if (!$isTextNode(marker)) throw new Error('expected marker text')
      marker.setTextContent('::fix revised')
    }, { discrete: true })

    const nodes = readText(editor)
    expect(nodes).toContainEqual({ text: '::fix revised', style: INDIANA_MARKER_STYLE, code: false })

    let exported = ''
    let copiedText = ''
    editor.read(() => {
      exported = $convertToMarkdownString(MARKDOWN_TRANSFORMERS)
      copiedText = $getRoot().getTextContent()
    })
    expect(exported).toBe('A paragraph ::fix revised')
    expect(copiedText).toBe('A paragraph ::fix revised')

    editor.update(() => {
      const paragraph = $getRoot().getFirstChild()
      if (!$isElementNode(paragraph)) throw new Error('expected paragraph')
      const marker = paragraph.getLastChild()
      if (!$isTextNode(marker)) throw new Error('expected marker text')
      marker.setTextContent('ordinary text')
    }, { discrete: true })

    expect(readText(editor)).toContainEqual({ text: 'A paragraph ordinary text', style: '', code: false })
  })

  it('removes marker presentation from Lexical clipboard data', () => {
    const data = stripMarkerStylesFromClipboard({
      'text/plain': 'A paragraph ::fix revised',
      'application/x-lexical-editor': JSON.stringify({
        namespace: 'casablanca-marker-highlight-test',
        nodes: [
          {
            type: 'text',
            version: 1,
            text: '::fix revised',
            style: INDIANA_MARKER_STYLE,
            children: []
          }
        ]
      })
    })
    const payload = JSON.parse(data['application/x-lexical-editor'] ?? '{}') as {
      nodes?: Array<{ style?: string }>
    }

    expect(data['text/plain']).toBe('A paragraph ::fix revised')
    expect(payload.nodes?.[0]?.style).toBe('')
  })

  it('leaves fenced and inline code markers alone', () => {
    const markdown = ['```', '::fix not a command', '```', '', '`::todo not a command`'].join('\n')
    const editor = makeEditor()

    editor.update(() => {
      $importMarkdown(markdown, MARKDOWN_TRANSFORMERS)
      styleMarkers($getRoot())
    }, { discrete: true })

    expect(readText(editor).filter((node) => node.text.includes('::'))).toEqual([
      { text: '::fix not a command', style: '', code: false },
      { text: '::todo not a command', style: '', code: true }
    ])
  })
})
