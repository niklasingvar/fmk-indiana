import type { CasablancaApi } from '@preload/index'

declare global {
  interface Window {
    api: CasablancaApi
  }
}

export {}
