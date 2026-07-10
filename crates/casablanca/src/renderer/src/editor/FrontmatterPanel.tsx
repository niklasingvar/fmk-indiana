import { useMemo, useState } from 'react'
import {
  addFrontmatterAnnotation,
  frontmatterAnnotations,
  frontmatterSource,
  projectFrontmatter,
  removeFrontmatterProperty,
  setFrontmatterProperty,
  wrapFrontmatter,
  type FrontmatterProperty,
  type FrontmatterScalar
} from '@shared/frontmatter'
import { INDIANA_COMMAND_OPTIONS, MarkerComposer } from '../MarkerComposer'

function PropertyInput({
  property,
  onChange
}: {
  property: FrontmatterProperty
  onChange: (value: FrontmatterScalar) => void
}) {
  if (typeof property.value === 'boolean') {
    return (
      <select
        value={String(property.value)}
        onChange={(e) => onChange(e.target.value === 'true')}
        className="min-w-0 flex-1 rounded border border-pane-border bg-pane-active px-2 py-1 text-xs outline-none focus:border-accent"
      >
        <option value="true">true</option>
        <option value="false">false</option>
      </select>
    )
  }

  return (
    <input
      type={typeof property.value === 'number' ? 'number' : 'text'}
      value={property.value ?? ''}
      placeholder={property.value === null ? 'null' : undefined}
      onChange={(e) => {
        if (typeof property.value === 'number') {
          const number = Number(e.target.value)
          if (Number.isFinite(number)) onChange(number)
        } else {
          onChange(e.target.value)
        }
      }}
      className="min-w-0 flex-1 rounded border border-pane-border bg-pane-active px-2 py-1 text-xs outline-none focus:border-accent"
    />
  )
}

export function FrontmatterPanel({
  frontmatter,
  onChange
}: {
  frontmatter: string
  onChange: (frontmatter: string) => void
}) {
  const projection = useMemo(() => projectFrontmatter(frontmatter), [frontmatter])
  const [rawMode, setRawMode] = useState(projection.kind === 'raw')
  const [annotating, setAnnotating] = useState<string | null>(null)
  const [newKey, setNewKey] = useState('')
  const [newValue, setNewValue] = useState('')
  const [error, setError] = useState('')
  const showRaw = rawMode || projection.kind === 'raw'

  const apply = (change: () => string): void => {
    try {
      onChange(change())
      setError('')
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  const addProperty = (): void => {
    apply(() => setFrontmatterProperty(frontmatter, newKey, newValue))
    if (newKey.trim() !== '') {
      setNewKey('')
      setNewValue('')
    }
  }

  return (
    <div className="flex h-full flex-col bg-pane">
      <div className="flex items-center justify-between border-b border-pane-border px-3 py-2">
        <div>
          <div className="text-xs font-medium text-text-strong">Properties</div>
          <div className="text-[11px] text-text-muted">YAML frontmatter</div>
        </div>
        <button
          onClick={() => setRawMode((raw) => !raw)}
          disabled={projection.kind === 'raw'}
          className="rounded border border-pane-border px-2 py-0.5 text-[11px] hover:bg-pane-hover disabled:opacity-50"
        >
          {showRaw ? 'Properties' : 'Raw YAML'}
        </button>
      </div>

      {showRaw ? (
        <div className="flex min-h-0 flex-1 flex-col p-3">
          {projection.kind === 'raw' && (
            <div className="mb-2 text-[11px] text-text-muted">{projection.reason}</div>
          )}
          <textarea
            value={frontmatterSource(frontmatter)}
            onChange={(e) => onChange(wrapFrontmatter(e.target.value))}
            spellCheck={false}
            className="min-h-0 flex-1 resize-none rounded border border-pane-border bg-code-bg p-3 font-mono text-xs leading-5 outline-none focus:border-accent"
          />
        </div>
      ) : (
        <div className="min-h-0 flex-1 overflow-auto p-3">
          {projection.properties.map((property) => {
            const annotations = frontmatterAnnotations(frontmatter, property.key)
            return (
              <div key={property.key} className="mb-3 rounded border border-pane-border bg-pane-active p-2">
                <div className="mb-1 flex items-center gap-1">
                  <span className="min-w-0 flex-1 truncate font-mono text-[11px] text-text-muted">
                    {property.key}
                  </span>
                  <button
                    onClick={() => setAnnotating((key) => (key === property.key ? null : property.key))}
                    title={`Comment on ${property.key}`}
                    className="rounded px-1 text-[11px] text-text-muted hover:bg-pane-hover hover:text-accent"
                  >
                    + ::
                  </button>
                  <button
                    onClick={() => apply(() => removeFrontmatterProperty(frontmatter, property.key))}
                    title={`Remove ${property.key}`}
                    className="rounded px-1 text-xs text-text-muted hover:bg-pane-hover hover:text-git-deleted"
                  >
                    ×
                  </button>
                </div>
                <PropertyInput
                  property={property}
                  onChange={(value) =>
                    apply(() => setFrontmatterProperty(frontmatter, property.key, value))
                  }
                />
                {annotations.length > 0 && (
                  <div className="mt-2 space-y-1">
                    {annotations.map((annotation, index) => (
                      <div
                        key={`${annotation}:${index}`}
                        className="rounded bg-code-bg px-2 py-1 font-mono text-[10px] text-text-muted"
                      >
                        {annotation}
                      </div>
                    ))}
                  </div>
                )}
                {annotating === property.key && (
                  <div className="mt-2 rounded border border-pane-border bg-pane p-2 shadow-lg">
                    <MarkerComposer
                      options={INDIANA_COMMAND_OPTIONS}
                      allowUnknown
                      onSubmit={(command) => {
                        apply(() => addFrontmatterAnnotation(frontmatter, property.key, command))
                        setAnnotating(null)
                      }}
                      onClose={() => setAnnotating(null)}
                    />
                  </div>
                )}
              </div>
            )
          })}

          <div className="rounded border border-dashed border-pane-border p-2">
            <div className="mb-1 text-[11px] text-text-muted">Add property</div>
            <input
              value={newKey}
              onChange={(e) => setNewKey(e.target.value)}
              placeholder="name"
              className="mb-1 w-full rounded border border-pane-border bg-pane-active px-2 py-1 font-mono text-xs outline-none focus:border-accent"
            />
            <div className="flex gap-1">
              <input
                value={newValue}
                onChange={(e) => setNewValue(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && addProperty()}
                placeholder="value"
                className="min-w-0 flex-1 rounded border border-pane-border bg-pane-active px-2 py-1 text-xs outline-none focus:border-accent"
              />
              <button
                onClick={addProperty}
                disabled={newKey.trim() === ''}
                className="rounded border border-pane-border px-2 py-1 text-xs hover:bg-pane-hover disabled:opacity-50"
              >
                Add
              </button>
            </div>
          </div>
        </div>
      )}

      {error !== '' && <div className="border-t border-pane-border px-3 py-2 text-xs text-git-deleted">{error}</div>}
    </div>
  )
}
