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

/** Claim status the daemon stamps into a marker's `[id:status]` bracket. */
export type MarkerStatus = 'working' | 'done' | 'failed'

/** One `::` marker found by `indiana scan`, path vault-relative. */
export interface VaultMarker {
  path: string
  /** 1-based line in the file (frontmatter included). */
  line: number
  kind: string
  /** The token as written, e.g. `::fix` or the alias `::f`. */
  rawToken: string
  message?: string
  /** Numeric batch label (`-1`, `-2`, …). Mutually exclusive with `agent`. */
  group?: number
  /** Named agent persona (`-m` / `-mike`), canonical name. */
  agent?: string
  id?: string
  status?: MarkerStatus
}

/** `available: false` = indiana binary missing or the scan failed. */
export interface VaultMarkersResult {
  available: boolean
  markers: VaultMarker[]
}

/**
 * Agent personas defined in the vault: the directory names under
 * `.indiana/agents/` that carry a `SYSTEM_PROMPT.md`. Sorted.
 */
export interface VaultAgentsResult {
  agents: string[]
}

/** Outcome of dispatching one batch (numeric group or agent persona). */
export interface DispatchResult {
  /** False when the batch was empty, the daemon is offline, or a turn runs. */
  accepted: boolean
  /** Number of markers claimed for this turn. */
  count: number
}

/** A live ACP agent process, projected by the Indiana daemon. */
export interface AgentJob {
  id: string
  root: string
  markers: string[]
  state: 'running' | 'awaiting_input'
  question: AgentQuestion | null
}

/** The one text field Casablanca currently renders for an agent question. */
export interface AgentQuestion {
  message: string
  field: string
}

export interface AgentJobsResult {
  online: boolean
  jobs: AgentJob[]
}

export type ElicitationAction = 'accept' | 'decline' | 'cancel'

export interface AnswerAgentJobResult {
  accepted: boolean
}

/** What one transcript entry represents in the job follow view. */
export type TranscriptEventKind = 'agent' | 'thought' | 'tool' | 'question' | 'answer'

/** One entry of a live turn's transcript; `seq` is monotonic per job. */
export interface TranscriptEvent {
  seq: number
  kind: TranscriptEventKind
  text: string
}

/**
 * A page of transcript events from `since_seq` on. `found: false` means the
 * job is gone (turn ended) — transcripts are daemon memory, like jobs.
 */
export interface JobTranscriptResult {
  found: boolean
  events: TranscriptEvent[]
  nextSeq: number
}

/** Chief of Staff queues (COS_PRD.md): agent tasks drain autonomously. */
export type CosQueue = 'agent' | 'human'

export type CosTaskState = 'open' | 'working' | 'done' | 'failed'

/** Where a captured task's marker lived at capture time — a jump hint. */
export interface CosTaskOrigin {
  path: string
  line: number
}

/** One line of `.indiana/chief-of-staff/tasks.md`, parsed by the Indiana CLI. */
export interface CosTask {
  id: string
  text: string
  queue: CosQueue
  state: CosTaskState
  origin?: CosTaskOrigin
  created?: string
}

/** `available: false` = indiana binary missing; the panel shows a hint. */
export interface CosTasksResult {
  available: boolean
  tasks: CosTask[]
}

/** One line of `.indiana/chief-of-staff/log.md` — the action log. */
export interface CosLogEntry {
  ts: string
  event: string
  id: string
  detail: string
}

export interface CosLogResult {
  available: boolean
  entries: CosLogEntry[]
}

/**
 * One durable agent-run audit record under `.indiana/chief-of-staff/runs/`
 * (COS_PRD.md), as emitted by `indiana runs --json` — the record grammar
 * lives in Rust only; this is a projection. `file` is the record's filename;
 * the full markdown is fetched on selection.
 */
export interface AgentRun {
  file: string
  job: string
  outcome: 'done' | 'failed' | string
  /** Stop reason or failure note, when the record carries one. */
  detail?: string
  /** `YYYY-MM-DD HH:MM` (UTC). */
  started: string
  ended: string
  root: string
  markers?: string[]
  tokensIn?: number
  tokensOut?: number
  tokensTotal?: number
  contextUsed?: number
  contextSize?: number
  cost?: number
  currency?: string
}

/** `available: false` = indiana binary missing or the listing failed. */
export interface AgentRunsResult {
  available: boolean
  runs: AgentRun[]
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

export type VaultTheme = 'light' | 'dark'

export type VaultState =
  | { status: 'unset' }
  | { status: 'ready'; rootPath: string; color: string; theme: VaultTheme }
  | { status: 'error'; message: string }
