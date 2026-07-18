/**
 * A tiny renderer-local bus so the editor (inline working spinner) can open
 * the TopBar's job follow popover without threading callbacks through
 * App → Shell → EditorPane → Editor → plugin.
 */

const bus = new EventTarget()
const OPEN = 'open-job-follow'

/** Ask the TopBar to open the follow view for a marker id. */
export function openJobFollow(markerId: string): void {
  bus.dispatchEvent(new CustomEvent(OPEN, { detail: markerId }))
}

export function onOpenJobFollow(cb: (markerId: string) => void): () => void {
  const listener = (event: Event): void => cb((event as CustomEvent<string>).detail)
  bus.addEventListener(OPEN, listener)
  return () => bus.removeEventListener(OPEN, listener)
}
