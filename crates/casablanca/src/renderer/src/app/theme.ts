/**
 * Theme switching: light is the default palette; `.dark` on <html> flips the
 * CSS variables in styles.css. Persisted per machine, applied before first
 * paint from main.tsx.
 */

export type Theme = 'light' | 'dark'

const STORAGE_KEY = 'casablanca:theme'

function apply(theme: Theme): void {
  document.documentElement.classList.toggle('dark', theme === 'dark')
}

export function initTheme(): Theme {
  const theme: Theme = localStorage.getItem(STORAGE_KEY) === 'dark' ? 'dark' : 'light'
  apply(theme)
  return theme
}

export function setTheme(theme: Theme): void {
  localStorage.setItem(STORAGE_KEY, theme)
  apply(theme)
}
