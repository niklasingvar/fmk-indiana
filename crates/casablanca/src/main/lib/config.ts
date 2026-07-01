import { app } from 'electron'
import { join } from 'node:path'
import { promises as fs } from 'node:fs'
import type { VaultConfig } from '@shared/domain'

const CONFIG_FILE = join(app.getPath('userData'), 'casablanca.config.json')

interface PersistedConfig {
  vaultRootPath?: string
}

async function readPersisted(): Promise<PersistedConfig> {
  try {
    const raw = await fs.readFile(CONFIG_FILE, 'utf8')
    return JSON.parse(raw) as PersistedConfig
  } catch {
    return {}
  }
}

async function writePersisted(cfg: PersistedConfig): Promise<void> {
  await fs.writeFile(CONFIG_FILE, JSON.stringify(cfg, null, 2), 'utf8')
}

export async function getVaultConfig(): Promise<VaultConfig | null> {
  const cfg = await readPersisted()
  return cfg.vaultRootPath ? { rootPath: cfg.vaultRootPath } : null
}

export async function setVaultConfig(vault: VaultConfig): Promise<void> {
  await writePersisted({ vaultRootPath: vault.rootPath })
}
