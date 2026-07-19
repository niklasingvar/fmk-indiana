import { promises as fs } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { FSWatcher } from 'chokidar'
import type { BrowserWindow } from 'electron'
import { IPC } from '@shared/ipc'
import { watchVault } from './watcher'

/**
 * Regression guard for the chokidar v4 upgrade: v4 dropped glob support, so
 * glob patterns like '**\/*.md' silently watch NOTHING — no error, no events.
 * External edits then never reach the renderer: the open note goes stale and
 * the tree stops refreshing. These tests run the real watcher against a real
 * temp folder, so a watch-setup regression fails loudly.
 */
describe('watchVault', () => {
  let root: string
  let watcher: FSWatcher
  let sent: Array<{ channel: string; payload: unknown }>

  const fakeWindow = {
    webContents: {
      send: (channel: string, payload: unknown) => sent.push({ channel, payload })
    }
  } as unknown as BrowserWindow

  const channels = (): string[] => sent.map((s) => s.channel)
  const payloadsOf = (channel: string): unknown[] =>
    sent.filter((s) => s.channel === channel).map((s) => s.payload)

  const start = async (): Promise<void> => {
    watcher = watchVault({ rootPath: root }, () => fakeWindow)
    await new Promise<void>((resolve) => watcher.on('ready', () => resolve()))
  }

  // awaitWriteFinish (200ms) + debounce (150ms) + fs latency.
  const waitFor = (assertion: () => void): Promise<void> =>
    vi.waitFor(assertion, { timeout: 4000, interval: 50 })

  beforeEach(async () => {
    // realpath: macOS tmpdir is a symlink (/var -> /private/var); the watcher
    // compares absolute paths against rootPath, so they must match.
    root = await fs.realpath(await fs.mkdtemp(join(tmpdir(), 'casa-watch-')))
    sent = []
  })

  afterEach(async () => {
    await watcher.close()
    // Drain the watcher's debounce timers (150ms) so a pending tree/git push
    // lands in this test's `sent`, not the next test's fresh one.
    await new Promise((resolve) => setTimeout(resolve, 400))
    await fs.rm(root, { recursive: true, force: true })
  })

  it('external edit to a markdown note pushes note:changed and tree:changed', async () => {
    await fs.mkdir(join(root, 'sub'))
    await fs.writeFile(join(root, 'sub', 'note.md'), 'original')
    await start()

    await fs.writeFile(join(root, 'sub', 'note.md'), 'edited by agent')

    await waitFor(() => {
      expect(payloadsOf(IPC.NOTE_CHANGED)).toContain('sub/note.md')
      expect(channels()).toContain(IPC.TREE_CHANGED)
    })
  })

  it('new markdown file pushes note:changed and tree:changed', async () => {
    await start()

    await fs.writeFile(join(root, 'fresh.md'), 'brand new')

    await waitFor(() => {
      expect(payloadsOf(IPC.NOTE_CHANGED)).toContain('fresh.md')
      expect(channels()).toContain(IPC.TREE_CHANGED)
    })
  })

  it('html edit pushes preview:changed', async () => {
    await fs.writeFile(join(root, 'page.html'), '<p>hi</p>')
    await start()

    await fs.writeFile(join(root, 'page.html'), '<p>edited</p>')

    await waitFor(() => {
      expect(payloadsOf(IPC.PREVIEW_CHANGED)).toContain('page.html')
      expect(channels()).toContain(IPC.TREE_CHANGED)
    })
  })

  it('settings.json edit pushes note:changed', async () => {
    await fs.mkdir(join(root, '.indiana', 'casablanca'), { recursive: true })
    await fs.writeFile(join(root, '.indiana', 'casablanca', 'settings.json'), '{}')
    await start()

    await fs.writeFile(join(root, '.indiana', 'casablanca', 'settings.json'), '{"theme":"dark"}')

    await waitFor(() => {
      expect(payloadsOf(IPC.NOTE_CHANGED)).toContain('.indiana/casablanca/settings.json')
      expect(channels()).toContain(IPC.TREE_CHANGED)
    })
  })

  it('irrelevant extensions and ignored dirs stay silent', async () => {
    await fs.mkdir(join(root, 'node_modules', 'pkg'), { recursive: true })
    await start()

    await fs.writeFile(join(root, 'data.txt'), 'not watched')
    await fs.writeFile(join(root, 'node_modules', 'pkg', 'readme.md'), 'ignored dir')

    // Longer than awaitWriteFinish + debounce: events would have arrived by now.
    await new Promise((resolve) => setTimeout(resolve, 800))
    expect(sent).toEqual([])
  })
})
