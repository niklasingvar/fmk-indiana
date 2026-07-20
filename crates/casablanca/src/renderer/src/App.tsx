import { useVault } from './storage/useVault'
import { Shell } from './app/Shell'

export function App() {
  if (typeof window !== 'undefined' && !window.api) {
    return (
      <div className="flex h-screen w-screen flex-col items-center justify-center gap-2 bg-pane p-6 text-center">
        <h1 className="text-xl font-semibold text-git-deleted">Casablanca runs in Electron, not the browser</h1>
        <p className="max-w-md text-sm text-text-muted">
          This URL is only the renderer dev server; <code>window.api</code> does not
          exist here. Close this tab and use the Electron window that{' '}
          <code>npm run dev</code> opens. If the Electron window itself shows this,
          the preload script failed — check the <code>preload</code> path in the main
          process and the Electron devtools console.
        </p>
      </div>
    )
  }

  const vault = useVault()
  return <Shell vault={vault} />
}
