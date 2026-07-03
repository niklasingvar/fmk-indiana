/**
 * Core domain model for Casablanca.
 *
 * The app is intentionally small: a vault is a folder on disk containing
 * markdown notes (and, later, inline Excalidraw diagrams). The tree is a
 * read projection of that folder; a note is the editable unit.
 */

export type NodeType = 'file' | 'folder'

export interface TreeNode {
  /** Stable path relative to the vault root, using '/' separators. */
  path: string
  name: string
  type: NodeType
  children?: TreeNode[]
}

export interface Note {
  /** Path relative to the vault root. */
  path: string
  name: string
  /** Raw markdown content (Excalidraw scenes embedded as fenced blocks). */
  content: string
  updatedAt: number
}

/**
 * A markdown file split into an opaque frontmatter block and the editable
 * body. The editor only ever sees the body; the frontmatter is carried
 * verbatim so autosave can never corrupt it. Parse/serialize live in
 * `note-serialization.ts` and are byte-stable by construction.
 */
export interface NoteDocument {
  /**
   * The raw frontmatter block — both `---` fences and the trailing newline
   * included — or null when the file has none. Opaque text, never parsed.
   */
  frontmatter: string | null
  /** Everything after the frontmatter block, verbatim. */
  body: string
}

/** Result of running `indiana copy` for the vault. */
export interface CopyAllResult {
  ok: boolean
  message: string
}

export interface VaultConfig {
  /** Absolute path to the vault folder on disk. */
  rootPath: string
}

export type VaultState =
  | { status: 'unset' }
  | { status: 'ready'; rootPath: string }
  | { status: 'error'; message: string }
