import { useEffect } from 'react'
import { useLexicalComposerContext } from '@lexical/react/LexicalComposerContext'
import { $getRoot, $getSelection, $isElementNode, $isRangeSelection, $isTextNode, type ElementNode, type TextNode } from 'lexical'
import type { MarkerClaimPatch } from '@shared/marker-claim'

export interface MarkerClaimSignal {
  /** Incremented per claim so identical patch lists still retrigger. */
  id: number
  patches: MarkerClaimPatch[]
}

interface Props {
  patch: MarkerClaimSignal | null
}

/** Contiguous runs of plain text children per element, depth-first. */
function collectTextRuns(root: ElementNode): TextNode[][] {
  const runs: TextNode[][] = []
  const stack: ElementNode[] = [root]
  while (stack.length > 0) {
    const element = stack.pop() as ElementNode
    let run: TextNode[] = []
    for (const child of element.getChildren()) {
      if ($isTextNode(child) && !child.hasFormat('code')) {
        run.push(child)
        continue
      }
      if (run.length > 0) runs.push(run)
      run = []
      if ($isElementNode(child)) stack.push(child)
    }
    if (run.length > 0) runs.push(run)
  }
  return runs
}

/** The minimal single-span edit turning `find` into `replace`. */
function spanEdit(find: string, replace: string): { at: number; deleteLen: number; insert: string } {
  let prefix = 0
  const max = Math.min(find.length, replace.length)
  while (prefix < max && find[prefix] === replace[prefix]) prefix++
  let suffix = 0
  while (
    suffix < max - prefix &&
    find[find.length - 1 - suffix] === replace[replace.length - 1 - suffix]
  ) {
    suffix++
  }
  return {
    at: prefix,
    deleteLen: find.length - prefix - suffix,
    insert: replace.slice(prefix, replace.length - suffix)
  }
}

function applyToRun(run: TextNode[], patch: MarkerClaimPatch): void {
  const { at, deleteLen, insert } = spanEdit(patch.find, patch.replace)
  // Locate the text node containing the edit offset.
  let offset = 0
  for (const node of run) {
    const length = node.getTextContentSize()
    if (at > offset + length || (at === offset + length && node !== run[run.length - 1])) {
      offset += length
      continue
    }
    const local = at - offset
    if (local + deleteLen <= length) {
      spliceKeepingSelection(node, local, deleteLen, insert)
    } else {
      // The edit spans node boundaries (should not happen for a bracket
      // inside one styled span) — rebuild the run wholesale as a fallback.
      run[0].setTextContent(patch.replace)
      for (const extra of run.slice(1)) extra.remove()
    }
    return
  }
}

/**
 * `spliceText` without stealing the caret: when the selection is anchored in
 * the spliced node at or past the edit, shift it by the length delta so the
 * user's cursor stays on the character it was on.
 */
function spliceKeepingSelection(node: TextNode, at: number, deleteLen: number, insert: string): void {
  const selection = $getSelection()
  const key = node.getKey()
  const delta = insert.length - deleteLen
  const shifted = (offset: number): number => (offset >= at + deleteLen ? offset + delta : offset)
  const anchor =
    $isRangeSelection(selection) && selection.anchor.getNode().getKey() === key
      ? shifted(selection.anchor.offset)
      : null
  const focus =
    $isRangeSelection(selection) && selection.focus.getNode().getKey() === key
      ? shifted(selection.focus.offset)
      : null
  node.spliceText(at, deleteLen, insert)
  if ($isRangeSelection(selection)) {
    if (anchor !== null) selection.anchor.set(key, anchor, 'text')
    if (focus !== null) selection.focus.set(key, focus, 'text')
  }
}

/**
 * Splices the Indiana daemon's marker-claim edit (`::fix -a …` →
 * `::fix[id:working] -a …`, later `:failed`) into the live document — no
 * remount, so cursor, undo history, and unsaved edits elsewhere survive.
 * The update is tagged `history-merge`: a claim must never be its own undo
 * step, or Cmd+Z would strip it, autosave would write the unclaimed marker
 * back, and the daemon would dispatch the same marker twice.
 */
export function MarkerClaimPlugin({ patch }: Props) {
  const [editor] = useLexicalComposerContext()

  useEffect(() => {
    if (!patch) return
    editor.update(
      () => {
        const runs = collectTextRuns($getRoot())
        for (const one of patch.patches) {
          const run = runs.find(
            (candidate) => candidate.map((n) => n.getTextContent()).join('') === one.find
          )
          if (!run) {
            // The user edited that exact line in the unsaved buffer — leave
            // it; disk still holds the claim (dirty-diverge fallback).
            console.warn('[marker-claim] no editor line matches claim:', one.find)
            continue
          }
          applyToRun(run, one)
        }
      },
      { tag: 'history-merge' }
    )
  }, [editor, patch?.id]) // eslint-disable-line react-hooks/exhaustive-deps

  return null
}
