import type { Config } from 'tailwindcss'

export default {
  content: ['./src/renderer/**/*.{ts,tsx,html}'],
  theme: {
    extend: {
      colors: {
        pane: 'rgb(var(--pane) / <alpha-value>)',
        'pane-border': 'rgb(var(--pane-border) / <alpha-value>)',
        'pane-hover': 'rgb(var(--pane-hover) / <alpha-value>)',
        'pane-active': 'rgb(var(--pane-active) / <alpha-value>)',
        'text-muted': 'rgb(var(--text-muted) / <alpha-value>)',
        'text-body': 'rgb(var(--text-body) / <alpha-value>)',
        'text-strong': 'rgb(var(--text-strong) / <alpha-value>)',
        'code-bg': 'rgb(var(--code-bg) / <alpha-value>)',
        'git-modified': 'rgb(var(--git-modified) / <alpha-value>)',
        'git-new': 'rgb(var(--git-new) / <alpha-value>)',
        'git-deleted': 'rgb(var(--git-deleted) / <alpha-value>)',
        accent: 'rgb(var(--accent) / <alpha-value>)'
      }
    }
  },
  plugins: []
} satisfies Config
