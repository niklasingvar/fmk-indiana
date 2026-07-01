import type { Config } from 'tailwindcss'

export default {
  content: ['./src/renderer/**/*.{ts,tsx,html}'],
  theme: {
    extend: {
      colors: {
        pane: 'rgb(var(--pane) / <alpha-value>)',
        'pane-border': 'rgb(var(--pane-border) / <alpha-value>)',
        'pane-hover': 'rgb(var(--pane-hover) / <alpha-value>)',
        'text-muted': 'rgb(var(--text-muted) / <alpha-value>)'
      }
    }
  },
  plugins: []
} satisfies Config
