import { app, BrowserWindow, shell } from 'electron'
import { join } from 'node:path'
import { registerIpc } from './ipc'
import { watchVault } from './watcher'
import { readTree } from './lib/vault'
import { IPC } from '@shared/ipc'

let mainWindow: BrowserWindow | null = null
let activeWatcher: ReturnType<typeof watchVault> | null = null

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
    void shell.openExternal(url)
    return { action: 'deny' }
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
  const vault = await registerIpc(mainWindow)

  if (vault) {
    activeWatcher = watchVault(vault, () => mainWindow)
    mainWindow.webContents.once('did-finish-load', async () => {
      try {
        mainWindow?.webContents.send(IPC.TREE_CHANGED, await readTree(vault))
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
