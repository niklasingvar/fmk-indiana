import type { ReactNode } from 'react'

/**
 * One stage chrome icon button. Presentational only — no vault or daemon access.
 */
export function StageIconButton({
  title,
  selected,
  disabled,
  onClick,
  children
}: {
  title: string
  selected: boolean
  disabled?: boolean
  onClick: () => void
  children: ReactNode
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      title={title}
      aria-label={title}
      aria-pressed={selected}
      className={`flex h-5 w-5 items-center justify-center rounded disabled:cursor-not-allowed disabled:opacity-40 ${
        selected ? 'bg-white/25' : 'bg-black/15 hover:bg-black/25'
      }`}
    >
      {children}
    </button>
  )
}
