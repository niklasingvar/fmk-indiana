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
