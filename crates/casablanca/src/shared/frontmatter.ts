const OPEN = '---\n'
const CLOSE = '---\n'

/** The YAML source between the two fences. */
export function frontmatterSource(block: string): string {
  if (!block.startsWith(OPEN) || !block.endsWith(CLOSE)) return block
  return block.slice(OPEN.length, -CLOSE.length)
}

/** Wrap editable YAML source in a byte-stable frontmatter fence. */
export function wrapFrontmatter(source: string): string {
  const yaml = source === '' || source.endsWith('\n') ? source : `${source}\n`
  return `${OPEN}${yaml}${CLOSE}`
}
