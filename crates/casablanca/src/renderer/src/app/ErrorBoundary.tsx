import { Component, type ErrorInfo, type ReactNode } from 'react'

interface State {
  error: Error | null
}

/** Surfaces renderer errors on screen instead of a silent black window. */
export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error('[renderer]', error, info.componentStack)
  }

  render(): ReactNode {
    if (this.state.error) {
      return (
        <div className="flex h-screen w-screen flex-col items-center justify-center gap-2 bg-pane p-6 text-center">
          <h1 className="text-xl font-semibold text-red-400">Renderer error</h1>
          <pre className="max-w-2xl overflow-auto rounded bg-black/40 p-3 text-left text-xs text-red-200">
            {this.state.error.message}
            {this.state.error.stack}
          </pre>
        </div>
      )
    }
    return this.props.children
  }
}
