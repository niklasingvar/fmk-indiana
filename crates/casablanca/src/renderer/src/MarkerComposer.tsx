import { useEffect, useMemo, useRef, useState } from 'react'
import { ANNOTATION_KINDS, type MessageContract } from '@shared/annotation-line'

export interface CommandOption {
  token: string
  label: string
  message: MessageContract
}

export const INDIANA_COMMAND_OPTIONS: readonly CommandOption[] = [
  ...ANNOTATION_KINDS,
  { token: 'action', label: 'Action', message: 'required' },
  { token: 'prompt', label: 'Prompt', message: 'optional' }
]

function splitCommand(text: string): { token: string; message: string } | null {
  const match = text.trim().match(/^::([A-Za-z]+|\?)(?:\s+(.*))?$/)
  if (!match) return null
  return { token: match[1], message: match[2]?.trim() ?? '' }
}

/**
 * Shared fast path for emitting a marker. Callers decide where the resulting
 * command text is stored; this component owns only the interaction.
 */
export function MarkerComposer({
  options,
  allowUnknown = false,
  onSubmit,
  onClose
}: {
  options: readonly CommandOption[]
  allowUnknown?: boolean
  onSubmit: (commandText: string) => void
  onClose: () => void
}) {
  const [command, setCommand] = useState('')
  const inputRef = useRef<HTMLInputElement | null>(null)
  const parsed = splitCommand(command)
  const selected = parsed ? options.find((option) => option.token === parsed.token) : undefined
  const known = selected !== undefined
  const canSubmit =
    parsed !== null &&
    (known || allowUnknown) &&
    (selected?.message !== 'required' || parsed.message !== '') &&
    !parsed.message.includes('::') &&
    !parsed.message.includes('`')

  const suggestions = useMemo(() => options.map((option) => `::${option.token}`), [options])

  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  const submit = (): void => {
    if (!parsed || !canSubmit) return
    const message = selected?.message === 'none' ? '' : parsed.message
    onSubmit(message === '' ? `::${parsed.token}` : `::${parsed.token} ${message}`)
  }

  const pick = (option: CommandOption): void => {
    if (option.message === 'none') {
      onSubmit(`::${option.token}`)
      return
    }
    setCommand(`::${option.token} `)
    requestAnimationFrame(() => inputRef.current?.focus())
  }

  return (
    <div onKeyDown={(e) => e.key === 'Escape' && onClose()}>
      <div className="flex flex-wrap gap-1">
        {options.map((option) => (
          <button
            key={option.token}
            onClick={() => pick(option)}
            className={`rounded border px-1.5 py-0.5 text-[11px] hover:bg-pane-hover ${
              selected?.token === option.token ? 'border-accent text-accent' : 'border-pane-border'
            }`}
          >
            {option.label}
          </button>
        ))}
      </div>
      <div className="mt-2 flex items-center gap-1">
        <input
          ref={inputRef}
          list="indiana-command-suggestions"
          value={command}
          onChange={(e) => setCommand(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && canSubmit) submit()
          }}
          placeholder="::fix message"
          className="min-w-0 flex-1 rounded border border-pane-border bg-pane-active px-2 py-1 text-xs outline-none focus:border-accent"
        />
        <datalist id="indiana-command-suggestions">
          {suggestions.map((suggestion) => (
            <option key={suggestion} value={suggestion} />
          ))}
        </datalist>
        <button
          disabled={!canSubmit}
          onClick={submit}
          className="rounded border border-pane-border px-2 py-1 text-xs hover:bg-pane-hover disabled:opacity-50"
        >
          Add
        </button>
      </div>
    </div>
  )
}
