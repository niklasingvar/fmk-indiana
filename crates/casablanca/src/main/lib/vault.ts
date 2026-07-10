import { promises as fs } from 'node:fs'
import { basename, join, relative, sep } from 'node:path'
import type { Note, TreeNode, VaultConfig } from '@shared/domain'

const NOTE_EXTENSION = '.md'
/** What the tree shows: notes plus previewable HTML documents. */
const TRACKED_EXTENSIONS = ['.md', '.html', '.htm']

function toPosix(p: string): string {
  return p.split(sep).join('/')
}

function isNote(name: string): boolean {
  return name.toLowerCase().endsWith(NOTE_EXTENSION)
}

function isTracked(name: string): boolean {
  const lower = name.toLowerCase()
  return TRACKED_EXTENSIONS.some((ext) => lower.endsWith(ext))
}

/** Read the vault into a sorted tree projection (folders first, then files). */
export async function readTree(vault: VaultConfig): Promise<TreeNode> {
  const rootName = basename(vault.rootPath) || vault.rootPath
  const root: TreeNode = { path: '', name: rootName, type: 'folder', children: [] }
  const folderByPath = new Map<string, TreeNode>([['', root]])

  const entries = await walk(vault.rootPath, '')
  entries.sort(compareNodes)

  for (const node of entries) {
    const parentPath = parentOf(node.path) ?? ''
    const parent = folderByPath.get(parentPath)
    const folder = node.type === 'folder' ? { ...node, children: [] } : node
    if (node.type === 'folder') folderByPath.set(node.path, folder as TreeNode)
    parent?.children?.push(folder)
  }

  return root
}

/** Heavy or derived folders the tree never shows; dotfolders like .indiana stay visible. */
const SKIP_DIRS = new Set(['.git', 'node_modules', 'target', 'dist', 'out'])
const SKIP_FILES = new Set(['.DS_Store'])

async function walk(root: string, rel: string): Promise<TreeNode[]> {
  const abs = join(root, rel)
  const out: TreeNode[] = []
  let dirents: import('node:fs').Dirent[]
  try {
    dirents = await fs.readdir(abs, { withFileTypes: true })
  } catch {
    return out
  }
  for (const d of dirents) {
    if (d.isDirectory() ? SKIP_DIRS.has(d.name) : SKIP_FILES.has(d.name)) continue
    const childRel = rel ? `${rel}/${d.name}` : d.name
    if (d.isDirectory()) {
      out.push({ path: childRel, name: d.name, type: 'folder' })
      out.push(...(await walk(root, childRel)))
    } else if (d.isFile() && isTracked(d.name)) {
      out.push({ path: childRel, name: d.name, type: 'file' })
    }
  }
  return out
}

function compareNodes(a: TreeNode, b: TreeNode): number {
  if (a.type !== b.type) return a.type === 'folder' ? -1 : 1
  return a.name.localeCompare(b.name, undefined, { numeric: true })
}

function parentOf(path: string): string | null {
  const idx = path.lastIndexOf('/')
  return idx === -1 ? null : path.slice(0, idx)
}

/** Read a note's content + stat. */
export async function readNote(vault: VaultConfig, rel: string): Promise<Note> {
  const abs = join(vault.rootPath, rel)
  const [content, stat] = await Promise.all([fs.readFile(abs, 'utf8'), fs.stat(abs)])
  return { path: rel, name: basename(rel), content, updatedAt: stat.mtimeMs }
}

/** Write a note (creates parent dirs if needed). */
export async function writeNote(vault: VaultConfig, rel: string, content: string): Promise<Note> {
  const abs = join(vault.rootPath, rel)
  await fs.mkdir(join(abs, '..'), { recursive: true })
  await fs.writeFile(abs, content, 'utf8')
  const stat = await fs.stat(abs)
  return { path: rel, name: basename(rel), content, updatedAt: stat.mtimeMs }
}

/** Create a new note with a default title. Returns the created note. */
export async function createNote(vault: VaultConfig, dirRel: string, name: string): Promise<Note> {
  const fileRel = join(dirRel, ensureMd(name)).split(sep).join('/')
  return writeNote(vault, fileRel, `# ${stripMd(name)}\n`)
}

export function toRelative(vault: VaultConfig, abs: string): string {
  return toPosix(relative(vault.rootPath, abs))
}

function ensureMd(name: string): string {
  return isNote(name) ? name : `${name}${NOTE_EXTENSION}`
}

function stripMd(name: string): string {
  return name.replace(/\.md$/i, '')
}
