import { app, BrowserWindow, shell } from 'electron'
import { join } from 'node:path'
import { registerIpc } from './ipc'
import { watchVault } from './watcher'
import { readTree } from './lib/vault'
import { gitStatus } from './lib/git'
import { registerVaultProtocol, registerVaultSchemeAsPrivileged } from './preview/protocol'
import { IPC } from '@shared/ipc'
import type { VaultConfig } from '@shared/domain'

let mainWindow: BrowserWindow | null = null
let activeWatcher: ReturnType<typeof watchVault> | null = null

// Must happen before app.whenReady() so vault:// is a standard scheme and
// relative asset URLs inside previewed HTML resolve through the handler.
registerVaultSchemeAsPrivileged()

function createWindow(): BrowserWindow {
  const win = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 720,
    show: false,
    autoHideMenuBar: true,
    backgroundColor: '#1e1e1e',
    webPreferences: {
      preload: join(__dirname, '../preload/index.mjs'),
      sandbox: false,
      contextIsolation: true,
      nodeIntegration: false
    }
  })

  win.on('ready-to-show', () => win.show())

  win.webContents.setWindowOpenHandler(({ url }) => {
    if (/^https?:/i.test(url)) void shell.openExternal(url)
    return { action: 'deny' }
  })

  // The window never navigates away from the app; external URLs go to the
  // OS browser (Track 2 security boundary).
  win.webContents.on('will-navigate', (e, url) => {
    e.preventDefault()
    if (/^https?:/i.test(url)) void shell.openExternal(url)
  })

  if (process.env['ELECTRON_RENDERER_URL']) {
    void win.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    void win.loadFile(join(__dirname, '../renderer/index.html'))
  }

  return win
}

app.whenReady().then(async () => {
  mainWindow = createWindow()

  // Re-point the watcher when the active project changes (or tear it down).
  const retargetWatcher = (vault: VaultConfig | null): void => {
    void activeWatcher?.close()
    activeWatcher = vault ? watchVault(vault, () => mainWindow) : null
  }

  const { vault, getVault } = await registerIpc(mainWindow, { retargetWatcher })
  registerVaultProtocol(getVault)

  if (vault) {
    activeWatcher = watchVault(vault, () => mainWindow)
    mainWindow.webContents.once('did-finish-load', async () => {
      try {
        mainWindow?.webContents.send(IPC.TREE_CHANGED, await readTree(vault))
        mainWindow?.webContents.send(IPC.GIT_CHANGED, await gitStatus(vault))
      } catch (err) {
        console.error('[main] initial tree push failed', err)
      }
    })
  }

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) mainWindow = createWindow()
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})

app.on('before-quit', async () => {
  await activeWatcher?.close()
})
