import { app } from 'electron'
import { join } from 'node:path'
import { promises as fs } from 'node:fs'
import type { Project, VaultConfig } from '@shared/domain'
import {
  normalizeConfig,
  addProjectToConfig,
  switchActive,
  setColor,
  removeProject as removeProjectPure,
  activeRecord,
  toProjectList,
  type PersistedConfig,
  type ProjectRecord
} from '@shared/projects'
import { readRepoColor, writeRepoSetting } from './repo-settings'

const CONFIG_FILE = join(app.getPath('userData'), 'casablanca.config.json')

/** Read + normalize (migrates the legacy single-vault shape on the way in). */
async function readConfig(): Promise<PersistedConfig> {
  try {
    const raw = await fs.readFile(CONFIG_FILE, 'utf8')
    return normalizeConfig(JSON.parse(raw))
  } catch {
    return normalizeConfig(null)
  }
}

async function writeConfig(cfg: PersistedConfig): Promise<void> {
  await fs.writeFile(CONFIG_FILE, JSON.stringify(cfg, null, 2), 'utf8')
}

/**
 * The per-repo `.indiana/casablanca/settings.json` `color` wins over the global
 * registry color when present (CASABLANCA_OVERVIEW.md) — so a committed color
 * travels with the repo and the `indiana casablanca set color` CLI is honored.
 * The global registry stays the fallback and drives palette de-duplication.
 */
async function withRepoColor(rec: ProjectRecord | null): Promise<ProjectRecord | null> {
  if (!rec) return rec
  const color = await readRepoColor(rec.rootPath)
  return color ? { ...rec, color } : rec
}

export async function listProjects(): Promise<Project[]> {
  const list = toProjectList(await readConfig())
  return Promise.all(
    list.map(async (p) => {
      const color = await readRepoColor(p.rootPath)
      return color ? { ...p, color } : p
    })
  )
}

export async function getActiveProject(): Promise<ProjectRecord | null> {
  return withRepoColor(activeRecord(await readConfig()))
}

/** Add (or re-activate) a project; returns the now-active record. */
export async function addProject(rootPath: string): Promise<ProjectRecord | null> {
  const cfg = addProjectToConfig(await readConfig(), rootPath)
  await writeConfig(cfg)
  return activeRecord(cfg)
}

export async function setActiveProject(rootPath: string): Promise<ProjectRecord | null> {
  const cfg = switchActive(await readConfig(), rootPath)
  await writeConfig(cfg)
  return activeRecord(cfg)
}

export async function setProjectColor(rootPath: string, color: string): Promise<void> {
  // Update the global registry (keeps palette de-dup working) and mirror the
  // color into the repo's own settings so it's committable and CLI-visible.
  await writeConfig(setColor(await readConfig(), rootPath, color))
  await writeRepoSetting(rootPath, 'color', color)
}

export async function removeProject(rootPath: string): Promise<ProjectRecord | null> {
  const cfg = removeProjectPure(await readConfig(), rootPath)
  await writeConfig(cfg)
  return activeRecord(cfg)
}

/** Convenience for callers that only need the active folder path. */
export async function getVaultConfig(): Promise<VaultConfig | null> {
  const active = await getActiveProject()
  return active ? { rootPath: active.rootPath } : null
}
