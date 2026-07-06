import { useVault } from './storage/useVault'
import { Shell } from './app/Shell'

export function App() {
  if (typeof window !== 'undefined' && !window.api) {
    return (
      <div className="flex h-screen w-screen flex-col items-center justify-center gap-2 bg-pane p-6 text-center">
        <h1 className="text-xl font-semibold text-git-deleted">Preload bridge missing</h1>
        <p className="max-w-md text-sm text-text-muted">
          <code>window.api</code> is not defined. The preload script did not load.
          Check the <code>preload</code> path in the main process and the Electron
          devtools console for errors.
        </p>
      </div>
    )
  }

  const vault = useVault()
  return <Shell vault={vault} />
}
