/**
 * Phase 4 — inline Excalidraw diagrams.
 *
 * NOT WIRED YET. This module will provide:
 *   - ExcalidrawNode: a Lexical DecoratorNode that lazy-loads
 *     `@excalidraw/excalidraw` and renders the canvas inline.
 *   - EXCALIDRAW_TRANSFORMER: serializes the scene as a fenced
 *     ` ```excalidraw ` block so it round-trips through markdown.
 *   - ExcalidrawPlugin: a toolbar command to insert a diagram node.
 *
 * Kept compiling (but unused) so phase 4 is pure wiring + implementation.
 */
import type { ReactElement } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import {
  DecoratorNode,
  type EditorConfig,
  type LexicalEditor,
  type NodeKey
} from 'lexical'
import type { ElementTransformer } from '@lexical/markdown'
import type { SerializedLexicalNode } from 'lexical'

interface ExcalidrawPayload extends SerializedLexicalNode {
  data: string
}

export class ExcalidrawNode extends DecoratorNode<ReactElement> {
  __data: string

  static getType(): string {
    return 'excalidraw'
  }

  static clone(node: ExcalidrawNode): ExcalidrawNode {
    return new ExcalidrawNode(node.__data, node.__key)
  }

  constructor(data: string, key?: NodeKey) {
    super(key)
    this.__data = data
  }

  createDOM(): HTMLElement {
    return document.createElement('div')
  }

  updateDOM(): false {
    return false
  }

  decorate(_editor: LexicalEditor, _config: EditorConfig): ReactElement {
    // Phase 4: lazy-load @excalidraw/excalidraw and render the canvas here.
    return (
      <div className="my-4 rounded border border-pane-border p-4 text-text-muted">
        Diagram (phase 4)
      </div>
    )
  }

  exportJSON(): ExcalidrawPayload {
    return { type: 'excalidraw', data: this.__data, version: 1 }
  }

  static importJSON(json: ExcalidrawPayload): ExcalidrawNode {
    return new ExcalidrawNode(json.data)
  }
}

export function $createExcalidrawNode(data = ''): ExcalidrawNode {
  return new ExcalidrawNode(data)
}

// Placeholder transformer — implemented in phase 4.
export const EXCALIDRAW_TRANSFORMER: ElementTransformer = {
  type: 'element',
  dependencies: [ExcalidrawNode],
  export: (): null => null,
  regExp: /```excalidraw\n([\s\S]*?)\n```/,
  replace: (): boolean => false
}

export function ExcalidrawPlugin(): null {
  const [editor] = useLexicalComposerContext()
  void editor
  return null
}
