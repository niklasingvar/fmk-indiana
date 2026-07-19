/**
 * Matching a raw markdown file line against a rendered editor block's text.
 * The markdown round-trip strips list bullets, heading hashes, and inline
 * formatting, so an exact match is tried first and two looser forms cover
 * the rest: the `::` marker tail (unique enough on its own) and plain
 * containment. Higher rank wins; 0 means no match.
 */
export function revealMatchRank(blockText: string, rawLine: string): number {
  const line = rawLine.trim()
  if (line.length === 0) return 0
  if (blockText === line) return 3
  const markerAt = line.indexOf('::')
  if (markerAt >= 0 && blockText.includes(line.slice(markerAt))) return 2
  if (blockText.includes(line)) return 1
  return 0
}
