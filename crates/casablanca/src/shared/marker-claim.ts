/**
 * Recognizes the Indiana daemon's marker-claim edit so the editor can adopt
 * it surgically instead of remounting: the only difference between the old
 * and new body is that one or more marker lines gained or changed an
 * `[id]`/`[id:status]` bracket right after the `::kind` token — the daemon's
 * `write::set_status` transform (`::fix -a msg` → `::fix[happy-otter:working]
 * -a msg`). Everything else (flags, message, other lines) is byte-identical.
 */

export interface MarkerClaimPatch {
  /** The exact full line as the editor currently has it. */
  find: string
  /** The exact full line as it now reads on disk. */
  replace: string
}

/** A claim bracket immediately after a marker kind token. */
const CLAIM_BRACKET = /(::[a-z?]+)\[[a-z0-9-]+(?::(?:working|done|failed))?\]/i

function stripClaimBracket(line: string): string {
  return line.replace(CLAIM_BRACKET, '$1')
}

/**
 * Apply claim patches to a body by exact full-line match (first occurrence).
 * A patch whose line was edited away in the meantime is skipped — the caller
 * degrades to the dirty-diverge path for that line.
 */
export function applyMarkerClaims(body: string, patches: MarkerClaimPatch[]): string {
  const lines = body.split('\n')
  for (const patch of patches) {
    const index = lines.indexOf(patch.find)
    if (index >= 0) lines[index] = patch.replace
  }
  return lines.join('\n')
}

/**
 * Line-level diff of two note bodies. Returns one patch per changed line when
 * every change is a claim-bracket insertion or replacement, else null (the
 * change is a real content edit and the caller falls back to full adoption).
 */
export function diffMarkerClaims(oldBody: string, newBody: string): MarkerClaimPatch[] | null {
  if (oldBody === newBody) return null
  const oldLines = oldBody.split('\n')
  const newLines = newBody.split('\n')
  if (oldLines.length !== newLines.length) return null

  const patches: MarkerClaimPatch[] = []
  for (let i = 0; i < oldLines.length; i++) {
    const before = oldLines[i]
    const after = newLines[i]
    if (before === after) continue
    if (!CLAIM_BRACKET.test(after)) return null
    if (stripClaimBracket(before) !== stripClaimBracket(after)) return null
    patches.push({ find: before, replace: after })
  }
  return patches.length > 0 ? patches : null
}
