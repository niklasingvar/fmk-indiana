/**
 * Tiny renderer-local bus so panels (e.g. the tasks panel's composer) can
 * append a `::` marker line through the live editor — one writer per open
 * note — without prop-drilling. Same shape as app/agents/job-events.ts.
 */
const bus = new EventTarget()

const APPEND = 'marker-append'

export function requestMarkerAppend(commandText: string): void {
  bus.dispatchEvent(new CustomEvent(APPEND, { detail: commandText }))
}

export function onMarkerAppend(cb: (commandText: string) => void): () => void {
  const listener = (e: Event): void => cb((e as CustomEvent<string>).detail)
  bus.addEventListener(APPEND, listener)
  return () => bus.removeEventListener(APPEND, listener)
}

/** A request to scroll a note's editor to the line holding a marker. */
export interface MarkerReveal {
  /** Vault-relative path of the note the marker lives in. */
  path: string
  /** The raw file line to locate in the editor. */
  lineText: string
}

const REVEAL = 'marker-reveal'

// The reveal is sticky, not just an event: opening another note remounts the
// editor, so the plugin that must scroll may not exist yet when the request
// fires. The pending value survives until the right note's plugin consumes it.
let pendingReveal: MarkerReveal | null = null

export function requestMarkerReveal(reveal: MarkerReveal): void {
  pendingReveal = reveal
  bus.dispatchEvent(new Event(REVEAL))
}

/** Take the pending reveal if it targets `path`; consuming clears it. */
export function consumeMarkerReveal(path: string): string | null {
  if (!pendingReveal || pendingReveal.path !== path) return null
  const { lineText } = pendingReveal
  pendingReveal = null
  return lineText
}

export function onMarkerReveal(cb: () => void): () => void {
  const listener = (): void => cb()
  bus.addEventListener(REVEAL, listener)
  return () => bus.removeEventListener(REVEAL, listener)
}
