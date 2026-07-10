/**
 * A slim full-width bar tinted with the active project's color — the ambient
 * "which project am I in" cue. The color comes from the `--project-color` CSS
 * variable (set in Shell from the active project), consumed here via `bg-project`.
 */
export function TopBar({ name }: { name: string }) {
  return (
    <header className="flex h-8 shrink-0 select-none items-center justify-center bg-project px-3 text-xs font-medium text-white/95">
      <span className="truncate drop-shadow-sm">{name}</span>
    </header>
  )
}
