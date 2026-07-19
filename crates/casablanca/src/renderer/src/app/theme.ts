/**
 * Theme application: light is the default palette; `.dark` on <html> flips the
 * CSS variables in styles.css. Source of truth is the active repo's
 * `.indiana/casablanca/settings.json` `theme` key — not a UI toggle.
 */

export type Theme = 'light' | 'dark'

export function applyTheme(theme: Theme): void {
  document.documentElement.classList.toggle('dark', theme === 'dark')
}

/** Default before a vault is ready (or when settings omit `theme`). */
export function initTheme(): Theme {
  applyTheme('light')
  return 'light'
}
