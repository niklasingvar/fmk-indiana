import { frontmatterSource, wrapFrontmatter } from '@shared/frontmatter'

export function FrontmatterPanel({
  frontmatter,
  onChange
}: {
  frontmatter: string
  onChange: (frontmatter: string) => void
}) {
  return (
    <div className="flex h-full flex-col bg-pane">
      <div className="flex items-center justify-between border-b border-pane-border px-3 py-2">
        <div>
          <div className="text-xs font-medium text-text-strong">Properties</div>
          <div className="text-[11px] text-text-muted">YAML frontmatter</div>
        </div>
      </div>

      <div className="flex min-h-0 flex-1 flex-col p-3">
        <textarea
          value={frontmatterSource(frontmatter)}
          onChange={(e) => onChange(wrapFrontmatter(e.target.value))}
          spellCheck={false}
          className="min-h-0 flex-1 resize-none rounded border border-pane-border bg-code-bg p-3 font-mono text-xs leading-5 outline-none focus:border-accent"
        />
      </div>
    </div>
  )
}
