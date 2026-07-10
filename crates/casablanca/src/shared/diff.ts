/**
 * Unified-diff parsing for the history panel. Pure text transform: git's
 * patch output in, typed display lines out. File headers (diff --git, index,
 * ---/+++, mode lines) are dropped — the panel already knows which file it
 * is showing. Hunk headers are kept as separators.
 */

export type DiffLineKind = 'added' | 'removed' | 'context' | 'hunk'

export interface DiffLine {
  kind: DiffLineKind
  /** Line content without the leading +/-/space marker. */
  text: string
}

export function parseUnifiedDiff(patch: string): DiffLine[] {
  const out: DiffLine[] = []
  let inHunk = false
  for (const line of patch.split('\n')) {
    if (line.startsWith('@@')) {
      inHunk = true
      out.push({ kind: 'hunk', text: line })
      continue
    }
    if (!inHunk) continue // file header region (diff --git, index, ---, +++)
    if (line.startsWith('+')) out.push({ kind: 'added', text: line.slice(1) })
    else if (line.startsWith('-')) out.push({ kind: 'removed', text: line.slice(1) })
    else if (line.startsWith(' ')) out.push({ kind: 'context', text: line.slice(1) })
    else if (line.startsWith('\\')) continue // "\ No newline at end of file"
    else inHunk = false // next file header (multi-file patch) or trailing blank
  }
  return out
}
