export function EmptyState({ onChoose }: { onChoose: () => void }) {
  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center gap-4 bg-pane text-center">
      <div className="text-5xl">🏜️</div>
      <h1 className="text-2xl font-semibold text-text-strong">Casablanca</h1>
      <p className="max-w-sm text-text-muted">
        A minimal note editor with a WYSIWYG editor and inline Excalidraw diagrams.
      </p>
      <button
        onClick={onChoose}
        className="mt-2 rounded-md bg-accent px-4 py-2 text-sm font-medium text-white hover:bg-accent/90"
      >
        Open a project folder
      </button>
    </div>
  )
}
