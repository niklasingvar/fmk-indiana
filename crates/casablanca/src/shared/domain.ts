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

/** Marker kinds offered by the HTML-preview annotation bubble. */
export type AnnotationKind =
  | 'question'
  | 'fix'
  | 'elaborate'
  | 'hate'
  | 'love'
  | 'keep'
  | 'delete'
  | 'note'
  | 'todo'

/** A single element annotation made in the HTML preview. */
export interface AnnotationRequest {
  /** Vault-relative posix path of the annotated HTML document. */
  docRelPath: string
  /** CSS selector for the element, computed by the injected annotator. */
  selector: string
  /** Short visible-text excerpt of the element. */
  excerpt: string
  kind: AnnotationKind
  /** User message; the contract per kind lives in `annotation-line.ts`. */
  message?: string
}

export interface AnnotationResult {
  /** Vault-relative path of the sidecar markdown file that received the line. */
  sidecarRelPath: string
}

export interface VaultConfig {
  /** Absolute path to the vault folder on disk. */
  rootPath: string
}

/**
 * A known project in the registry, projected for the renderer. `color` is an
 * "r g b" triple (see `projects.ts`); `name` is the folder's last segment.
 */
export interface Project {
  rootPath: string
  name: string
  color: string
  active: boolean
}

/** Simplified git working-tree state, used to tint tree rows. */
export type GitFileStatus = 'modified' | 'new' | 'deleted'

/** Vault-relative path → status; folders carry their children's aggregate. */
export type GitStatusMap = Record<string, GitFileStatus>

/** One commit touching a file, for the per-note history panel. */
export interface GitLogEntry {
  hash: string
  /** Commit time in epoch milliseconds. */
  timestamp: number
  /** Commit subject — by loop convention `<command> | <target> — outcome`. */
  subject: string
}

export type VaultState =
  | { status: 'unset' }
  | { status: 'ready'; rootPath: string; color: string }
  | { status: 'error'; message: string }
