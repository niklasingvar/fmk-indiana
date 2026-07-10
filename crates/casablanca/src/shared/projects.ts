/**
 * Pure project-registry logic: the list of known project folders, which one is
 * active, and each one's identity color. No electron and no fs live here, so it
 * can be unit-tested directly — `lib/config.ts` (which imports `app` from
 * electron at module load) cannot. `config.ts` composes these transforms with
 * disk I/O. Mirrors the codebase's pure-helper convention (`flatten-tree.ts`,
 * `resolve-link.ts`).
 *
 * Colors are stored as space-separated RGB triples ("r g b") to match every
 * other token in `styles.css`, so `rgb(var(--project-color))` and inline
 * `rgb(${color})` both work.
 */

import type { Project } from './domain'

/**
 * Curated identity palette. Mid-tone and saturated so a near-white foreground
 * reads well on the top bar in both light and dark themes.
 */
export const PROJECT_PALETTE: string[] = [
  '217 70 66', // red
  '223 120 44', // orange
  '191 145 30', // amber
  '76 152 76', // green
  '44 150 150', // teal
  '59 122 214', // blue
  '126 92 200', // purple
  '196 84 148' // magenta
]

export interface ProjectRecord {
  /** Absolute path to the project folder. */
  rootPath: string
  /** Identity color as an "r g b" triple. */
  color: string
}

export interface PersistedConfig {
  projects: ProjectRecord[]
  activePath: string | null
}

/** Raw JSON as read from disk — may be a legacy single-vault shape or empty. */
interface RawConfig {
  projects?: ProjectRecord[]
  activePath?: string
  /** Legacy: the single vault root, pre-multi-project. */
  vaultRootPath?: string
}

/** First palette entry not already taken; wraps by count once all are used. */
export function pickColor(used: string[]): string {
  const free = PROJECT_PALETTE.find((c) => !used.includes(c))
  return free ?? PROJECT_PALETTE[used.length % PROJECT_PALETTE.length]
}

/** Last path segment, tolerating trailing slashes and either separator. */
export function projectName(rootPath: string): string {
  const cleaned = rootPath.replace(/[/\\]+$/, '')
  const idx = Math.max(cleaned.lastIndexOf('/'), cleaned.lastIndexOf('\\'))
  return idx === -1 ? cleaned : cleaned.slice(idx + 1)
}

/**
 * Coerce raw JSON into the current shape: migrate a legacy `vaultRootPath` into
 * a one-entry project list, dedupe by path, assign colors to any that lack one,
 * and ensure `activePath` points at a real project (or null).
 */
export function normalizeConfig(raw: RawConfig | null | undefined): PersistedConfig {
  const projects: ProjectRecord[] = []
  const seen = new Set<string>()
  const add = (rootPath: string, color?: string): void => {
    if (!rootPath || seen.has(rootPath)) return
    seen.add(rootPath)
    projects.push({ rootPath, color: color || pickColor(projects.map((p) => p.color)) })
  }

  for (const p of raw?.projects ?? []) add(p?.rootPath, p?.color)
  if (raw?.vaultRootPath) add(raw.vaultRootPath)

  let activePath = raw?.activePath ?? raw?.vaultRootPath ?? projects[0]?.rootPath ?? null
  if (activePath && !seen.has(activePath)) activePath = projects[0]?.rootPath ?? null
  return { projects, activePath }
}

/** Add a project (dedupe by path) and make it active. */
export function addProjectToConfig(cfg: PersistedConfig, rootPath: string): PersistedConfig {
  if (cfg.projects.some((p) => p.rootPath === rootPath)) {
    return { ...cfg, activePath: rootPath }
  }
  const color = pickColor(cfg.projects.map((p) => p.color))
  return { projects: [...cfg.projects, { rootPath, color }], activePath: rootPath }
}

/** Make an existing project active; a no-op if the path is unknown. */
export function switchActive(cfg: PersistedConfig, rootPath: string): PersistedConfig {
  if (!cfg.projects.some((p) => p.rootPath === rootPath)) return cfg
  return { ...cfg, activePath: rootPath }
}

/** Recolor one project. */
export function setColor(cfg: PersistedConfig, rootPath: string, color: string): PersistedConfig {
  return {
    ...cfg,
    projects: cfg.projects.map((p) => (p.rootPath === rootPath ? { ...p, color } : p))
  }
}

/** Drop a project; if it was active, fall back to the first remaining one. */
export function removeProject(cfg: PersistedConfig, rootPath: string): PersistedConfig {
  const projects = cfg.projects.filter((p) => p.rootPath !== rootPath)
  const activePath =
    cfg.activePath === rootPath ? (projects[0]?.rootPath ?? null) : cfg.activePath
  return { projects, activePath }
}

/** The active record, or null when nothing is active. */
export function activeRecord(cfg: PersistedConfig): ProjectRecord | null {
  return cfg.projects.find((p) => p.rootPath === cfg.activePath) ?? null
}

/** Projection for the renderer: adds a display name and an active flag. */
export function toProjectList(cfg: PersistedConfig): Project[] {
  return cfg.projects.map((p) => ({
    rootPath: p.rootPath,
    name: projectName(p.rootPath),
    color: p.color,
    active: p.rootPath === cfg.activePath
  }))
}
