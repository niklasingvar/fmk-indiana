import { useEffect } from 'react'
import {
  $getClipboardDataFromSelection,
  setLexicalClipboardDataTransfer,
  type LexicalClipboardData
} from '@lexical/clipboard'
import { $isCodeNode } from '@lexical/code'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import {
  $getSelection,
  $isElementNode,
  $isTextNode,
  COMMAND_PRIORITY_HIGH,
  COPY_COMMAND,
  TextNode
} from 'lexical'

/**
 * Presentation-only marker styling. Indiana remains responsible for parsing
 * and compiling these commands; this plugin only makes their visible suffix
 * easier to scan in the editor.
 */
export const INDIANA_MARKER_STYLE =
  'background-color: rgb(var(--marker-bg)); color: rgb(var(--marker-text)); text-decoration: underline; text-decoration-color: rgb(var(--marker-border)); text-decoration-thickness: 2px; text-underline-offset: 2px;'

const MARKER_PATTERN =
  /::(?:question|q|\?|hate|h|love|l|keep|k|fix|f|elaborate|e|note|n|action|a|todo|td|delete|d|prompt|p)(?:\[[^\]\n]*\])?(?:\s+[^\n]*)?$/i

function markerStart(text: string): number | null {
  return text.match(MARKER_PATTERN)?.index ?? null
}

const MARKER_STYLE_PROPERTIES = [
  'background-color',
  'color',
  'text-decoration',
  'text-decoration-color',
  'text-decoration-thickness',
  'text-underline-offset'
] as const

function stripMarkerStyleFromHtml(html: string): string {
  if (typeof DOMParser === 'undefined') return html

  const document = new DOMParser().parseFromString(html, 'text/html')
  for (const element of Array.from(document.querySelectorAll<HTMLElement>('[style]'))) {
    if (!(element.getAttribute('style') ?? '').includes('--marker-bg')) continue
    for (const property of MARKER_STYLE_PROPERTIES) element.style.removeProperty(property)
    if (!element.getAttribute('style')?.trim()) element.removeAttribute('style')
  }
  return document.body.innerHTML
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function stripMarkerStyleFromLexicalJson(serialized: string): string {
  try {
    const payload: unknown = JSON.parse(serialized)
    const visit = (value: unknown): void => {
      if (!isRecord(value)) return
      if (value.style === INDIANA_MARKER_STYLE) value.style = ''
      for (const key of ['children', 'nodes']) {
        const children = value[key]
        if (Array.isArray(children)) children.forEach(visit)
      }
    }
    visit(payload)
    return JSON.stringify(payload)
  } catch {
    return serialized
  }
}

export function stripMarkerStylesFromClipboard(data: LexicalClipboardData): LexicalClipboardData {
  const clean = { ...data }
  if (clean['text/html'] !== undefined) clean['text/html'] = stripMarkerStyleFromHtml(clean['text/html'])
  if (clean['application/x-lexical-editor'] !== undefined) {
    clean['application/x-lexical-editor'] = stripMarkerStyleFromLexicalJson(clean['application/x-lexical-editor'])
  }
  return clean
}

function hasClipboardData(
  event: ClipboardEvent | KeyboardEvent | null
): event is ClipboardEvent {
  return event !== null && 'clipboardData' in event && event.clipboardData !== null
}

function copyWithoutMarkerStyles(event: ClipboardEvent | KeyboardEvent | null): boolean {
  if (!hasClipboardData(event) || $getSelection() === null) return false
  const clipboardData = event.clipboardData
  if (clipboardData === null) return false
  event.preventDefault()
  setLexicalClipboardDataTransfer(
    clipboardData,
    stripMarkerStylesFromClipboard($getClipboardDataFromSelection())
  )
  return true
}

function contiguousTextSiblings(node: TextNode): TextNode[] {
  const parent = node.getParent()
  if (parent === null || !$isElementNode(parent)) return [node]

  const children = parent.getChildren()
  const index = children.indexOf(node)
  if (index < 0) return [node]

  const isPlainText = (child: (typeof children)[number]): child is TextNode =>
    $isTextNode(child) && !child.hasFormat('code')
  const group: TextNode[] = [node]

  for (let i = index - 1; i >= 0; i--) {
    const child = children[i]
    if (!isPlainText(child)) break
    group.unshift(child)
  }
  for (let i = index + 1; i < children.length; i++) {
    const child = children[i]
    if (!isPlainText(child)) break
    group.push(child)
  }
  return group
}

function clearMarkerStyle(node: TextNode): void {
  if (node.getStyle() === INDIANA_MARKER_STYLE) node.setStyle('')
}

/**
 * Apply or remove our style without changing the text or its markdown export.
 * Lexical may split a command while it is being typed, so the scan covers
 * contiguous text siblings rather than only the dirty node.
 */
export function highlightMarker(node: TextNode): void {
  const parent = node.getParent()
  if (node.hasFormat('code') || (parent !== null && $isCodeNode(parent))) return

  const group = contiguousTextSiblings(node)
  const text = group.map((child) => child.getTextContent()).join('')
  const start = markerStart(text)
  if (start === null) {
    group.forEach(clearMarkerStyle)
    return
  }

  let offset = 0
  for (const child of group) {
    const length = child.getTextContentSize()
    const childEnd = offset + length
    if (childEnd <= start) {
      clearMarkerStyle(child)
    } else if (offset < start) {
      const originalStyle = child.getStyle()
      const [, markerNode] = child.splitText(start - offset)
      if (originalStyle === INDIANA_MARKER_STYLE) child.setStyle('')
      markerNode.setStyle(INDIANA_MARKER_STYLE)
    } else if (child.getStyle() !== INDIANA_MARKER_STYLE) {
      child.setStyle(INDIANA_MARKER_STYLE)
    }
    offset = childEnd
  }
}

export function MarkerHighlightPlugin() {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    const unregisterTransform = editor.registerNodeTransform(TextNode, highlightMarker)
    const unregisterCopy = editor.registerCommand(COPY_COMMAND, copyWithoutMarkerStyles, COMMAND_PRIORITY_HIGH)
    return () => {
      unregisterTransform()
      unregisterCopy()
    }
  }, [editor])

  return null
}
