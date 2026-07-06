import { describe, expect, it } from 'vitest'
import { createHeadlessEditor } from '@lexical/headless'
import { $convertToMarkdownString } from '@lexical/markdown'
import { HeadingNode, QuoteNode } from '@lexical/rich-text'
import { ListNode, ListItemNode } from '@lexical/list'
import { $isLinkNode, LinkNode } from '@lexical/link'
import { CodeNode, CodeHighlightNode } from '@lexical/code'
import { TableCellNode, TableNode, TableRowNode } from '@lexical/table'
import { $getRoot, $isElementNode, type LexicalNode } from 'lexical'
import { $importMarkdown, MARKDOWN_TRANSFORMERS } from './plugins/MarkdownPlugin'

/**
 * Marker safety: `::` marker lines must survive markdown → Lexical → markdown.
 * The editor is pointed at repos whose files carry Indiana markers; eating or
 * mangling one corrupts the review loop. Uses the same node set and
 * transformers as the real editor (Editor.tsx).
 */

function makeEditor() {
  return createHeadlessEditor({
    namespace: 'casablanca-test',
    nodes: [HeadingNode, QuoteNode, ListNode, ListItemNode, LinkNode, CodeNode, CodeHighlightNode, TableNode, TableRowNode, TableCellNode],
    onError: (error: Error) => {
      throw error
    }
  })
}

function roundTrip(markdown: string): string {
  const editor = makeEditor()
  editor.update(
    () => {
      $importMarkdown(markdown, MARKDOWN_TRANSFORMERS)
    },
    { discrete: true }
  )
  let out = ''
  editor.read(() => {
    out = $convertToMarkdownString(MARKDOWN_TRANSFORMERS)
  })
  return out
}

function countLinks(markdown: string): number {
  const editor = makeEditor()
  editor.update(
    () => {
      $importMarkdown(markdown, MARKDOWN_TRANSFORMERS)
    },
    { discrete: true }
  )
  let count = 0
  editor.read(() => {
    const walk = (node: LexicalNode): void => {
      if ($isLinkNode(node)) count++
      if ($isElementNode(node)) node.getChildren().forEach(walk)
    }
    walk($getRoot())
  })
  return count
}

describe('marker tokens survive the round-trip', () => {
  // One line per marker kind, long and short forms, in the positions markers
  // actually appear: end of a paragraph line, headings, list items, quotes.
  const markerLines = [
    'A paragraph that needs work ::fix make it shorter',
    'Something unclear here ::question',
    'Something unclear here ::q why is this async?',
    'This line is wrong ::hate',
    'This line is great ::love apply the pattern',
    'Never touch this ::keep',
    'Context for the agent ::note remember the constraint',
    'Do the thing ::action refactor the parser',
    'Later ::todo write the tests',
    'Remove this section ::delete',
    'Run it ::prompt scaffold the deck',
    'Act on this ::elaborate expand the argument'
  ]

  for (const line of markerLines) {
    it(JSON.stringify(line), () => {
      expect(roundTrip(line)).toBe(line)
    })
  }

  it('markers in headings, lists, and quotes', () => {
    const doc = [
      '# Title needs punch ::fix punch it up',
      '',
      '- first item ::hate',
      '- second item ::love keep this pattern',
      '',
      '> quoted claim ::q source?'
    ].join('\n')
    expect(roundTrip(doc)).toBe(doc)
  })

  it('marker inside a code fence stays verbatim', () => {
    const doc = ['```rust', 'let x = 1; // ::fix not a real marker here', '```'].join('\n')
    expect(roundTrip(doc)).toBe(doc)
  })

  it('checklist lines keep their markers', () => {
    const doc = ['- [ ] open task ::todo break this down', '- [x] done task ::keep'].join('\n')
    const out = roundTrip(doc)
    expect(out).toContain('::todo break this down')
    expect(out).toContain('::keep')
  })

  it('GFM tables round-trip stably', () => {
    const table = [
      '| Category | Count | Examples |',
      '| --- | --- | --- |',
      '| flags | 5 | triangular-flag, crossed-flags |',
      '| geography | 8 | globe, compass, world-map |'
    ].join('\n')
    expect(roundTrip(table)).toBe(table)
  })
})

describe('document round-trip stability (canonical form)', () => {
  // Whole-document fixture in Lexical's canonical output form: single blank
  // lines between blocks, `-` bullets, no trailing newline. Content in this
  // form must be byte-stable — that is what autosave writes back.
  it('a realistic doc is byte-stable', () => {
    const doc = [
      '# ARCHITECTURE ::fix rename the heading',
      '',
      'First paragraph with a marker at the end ::note this matters',
      '',
      '## Components',
      '',
      '- indiana core ::keep',
      '- the daemon ::q why a socket?',
      '',
      '> One write chokepoint mutates user files. ::love',
      '',
      '```sh',
      'indiana copy .',
      '```'
    ].join('\n')
    expect(roundTrip(doc)).toBe(doc)
  })

  // Known, accepted normalizations (documented, not bugs): the editor emits
  // no trailing newline and collapses runs of blank lines. Markers survive.
  it('normalization keeps marker content intact', () => {
    const doc = 'line one ::fix tighten\n\n\n\nline two ::hate\n'
    const out = roundTrip(doc)
    expect(out).toContain('line one ::fix tighten')
    expect(out).toContain('line two ::hate')
  })
})

describe('links import as LinkNodes and round-trip', () => {
  it('plain links stay links and byte-stable', () => {
    const doc = 'see [the spec](./docs/spec.md) for details'
    expect(roundTrip(doc)).toBe(doc)
    expect(countLinks(doc)).toBe(1)
  })

  it('links with inline-code text become links (merge pass) and round-trip', () => {
    const doc = 'see [`ANNOTATE.md`](./ANNOTATE.md) for classes'
    expect(roundTrip(doc)).toBe(doc)
    expect(countLinks(doc)).toBe(1)
  })

  it('merges code-text links inside table cells', () => {
    const table = [
      '| I want to... | Go to |',
      '| --- | --- |',
      '| Annotate | [`review/annotate.html`](./review/annotate.html) |'
    ].join('\n')
    expect(roundTrip(table)).toBe(table)
    expect(countLinks(table)).toBe(1)
  })

  it('leaves near-misses untouched', () => {
    const doc = 'brackets [ `code` ] (not-a-link) and `code`](dangling)'
    expect(roundTrip(doc)).toBe(doc)
    expect(countLinks(doc)).toBe(0)
  })
})
