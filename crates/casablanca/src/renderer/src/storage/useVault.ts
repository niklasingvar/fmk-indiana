import { useCallback, useEffect, useRef, useState } from 'react'
import type { GitStatusMap, Note, NoteDocument, Project, TreeNode, VaultState } from '@shared/domain'
import { isHtmlPath } from '@shared/annotation-line'
import { applyMarkerClaims, diffMarkerClaims, type MarkerClaimPatch } from '@shared/marker-claim'
import { parseNoteDocument, serializeNoteDocument } from '@shared/note-serialization'
import { applyTheme } from '../app/theme'

const SETTINGS_REL = '.indiana/casablanca/settings.json'

const AUTOSAVE_MS = 500

/**
 * Orchestrates vault + tree + the currently open note. All filesystem access
 * goes through `window.api`; the renderer never touches disk directly.
 *
 * The draft is a NoteDocument, not a raw string: the file is parsed once when
 * a note opens and serialized once on autosave. The Lexical editor only ever
 * sees `draft.body` — frontmatter rides along verbatim and cannot be
 * corrupted by the markdown round-trip.
 */
export function useVault() {
  const [vaultState, setVaultState] = useState<VaultState>({ status: 'unset' })
  const [projects, setProjects] = useState<Project[]>([])
  const [tree, setTree] = useState<TreeNode | null>(null)
  const [gitStatus, setGitStatus] = useState<GitStatusMap>({})
  const [activeNote, setActiveNote] = useState<Note | null>(null)
  const [draft, setDraft] = useState<NoteDocument | null>(null)
  const [saving, setSaving] = useState(false)
  // Bumped when an external edit is adopted into the open note, so the editor
  // remounts on fresh content (its Lexical state is seeded once per key).
  const [noteVersion, setNoteVersion] = useState(0)
  // The daemon's marker-claim edit, published for the editor to splice in
  // place (no remount, works on dirty buffers). `id` retriggers the effect
  // when successive claims produce identical patch lists.
  const [markerPatch, setMarkerPatch] = useState<{
    id: number
    patches: MarkerClaimPatch[]
  } | null>(null)
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const savePromise = useRef<Promise<void> | null>(null)
  // Live mirrors for the note:changed subscription (one stable listener).
  const activeNoteRef = useRef<Note | null>(null)
  const draftRef = useRef<NoteDocument | null>(null)
  activeNoteRef.current = activeNote
  draftRef.current = draft

  const refreshProjects = useCallback(async () => {
    setProjects(await window.api.projects.list())
  }, [])

  // Load persisted vault + project list on mount.
  useEffect(() => {
    void window.api.vault.get().then((s) => setVaultState(s))
    void refreshProjects()
  }, [refreshProjects])

  // Theme is a per-repo settings.json key — apply whenever vault state carries it.
  useEffect(() => {
    if (vaultState.status === 'ready') applyTheme(vaultState.theme)
  }, [vaultState])

  // Re-read vault state when settings.json changes (in-app save or external edit).
  useEffect(() => {
    if (vaultState.status !== 'ready') return
    return window.api.notes.onChanged((rel) => {
      if (rel !== SETTINGS_REL) return
      void window.api.vault.get().then((s) => {
        if (s.status === 'ready') setVaultState(s)
      })
    })
  }, [vaultState.status])

  // Subscribe to tree changes (covers both initial load and external edits).
  useEffect(() => {
    if (vaultState.status !== 'ready') return
    const off = window.api.tree.onChanged((t) => setTree(t))
    void window.api.tree.read().then(setTree)
    return off
  }, [vaultState])

  // Git working-tree status pushed alongside every tree refresh.
  useEffect(() => {
    if (vaultState.status !== 'ready') return
    return window.api.git.onChanged(setGitStatus)
  }, [vaultState])

  // Adopt external on-disk edits to the OPEN note — the Indiana daemon's
  // marker claims (`::fix -a …` → `::fix[id:working] -a …`) and the agent's
  // fixes. Without this the buffer goes stale: the user never sees the loop
  // run, and worse, autosaving the stale buffer wipes the claim and the same
  // marker is dispatched again (duplicate agent turns and commits).
  useEffect(() => {
    if (vaultState.status !== 'ready') return
    return window.api.notes.onChanged((rel) => {
      const note = activeNoteRef.current
      if (!note || note.path !== rel) return
      void window.api.notes.read(rel).then((fresh) => {
        const current = activeNoteRef.current
        if (!current || current.path !== rel) return
        // Our own autosave echo — already baselined by persistNote.
        if (fresh.content === current.content) return
        const liveDraft = draftRef.current
        const serialized = liveDraft ? serializeNoteDocument(liveDraft) : null
        // Disk caught up to the live draft (own write racing keystrokes):
        // re-baseline only, keep the editor untouched.
        if (serialized !== null && serialized === fresh.content) {
          setActiveNote(fresh)
          return
        }
        // The daemon's marker claim (`::fix -a …` → `::fix[id:working] -a …`,
        // or a later `:failed` flip): splice it into the live editor instead
        // of remounting — cursor, undo, and unsaved edits elsewhere survive.
        // Runs on dirty buffers too; that is the point: the claim must land
        // before the next autosave so it is never clobbered (double dispatch).
        const oldDoc = parseNoteDocument(current.content)
        const freshDoc = parseNoteDocument(fresh.content)
        if (oldDoc.frontmatter === freshDoc.frontmatter) {
          const patches = diffMarkerClaims(oldDoc.body, freshDoc.body)
          if (patches) {
            setActiveNote(fresh)
            setDraft(liveDraft ? { ...liveDraft, body: applyMarkerClaims(liveDraft.body, patches) } : freshDoc)
            setMarkerPatch((prev) => ({ id: (prev?.id ?? 0) + 1, patches }))
            return
          }
        }
        // Clean buffer → adopt the external edit wholesale.
        if (serialized === null || serialized === current.content) {
          setActiveNote(fresh)
          setDraft(parseNoteDocument(fresh.content))
          setNoteVersion((v) => v + 1)
          return
        }
        // Dirty buffer + diverged disk: adopting would drop keystrokes,
        // saving would clobber the agent. Keep the user's text; the next
        // autosave wins. Follow-up: three-way merge (shared/diff).
        console.warn('[vault] external change to a dirty note left on disk:', rel)
      })
    })
  }, [vaultState])

  const persistNote = useCallback(async (path: string, content: string): Promise<void> => {
    const saved = await window.api.notes.write(path, content)
    setActiveNote((prev) => (prev ? { ...prev, content: saved.content, updatedAt: saved.updatedAt } : prev))
  }, [])

  // Debounced autosave: when the serialized draft diverges from the saved
  // note, persist.
  useEffect(() => {
    if (!activeNote || !draft) return
    const serialized = serializeNoteDocument(draft)
    if (serialized === activeNote.content) return
    setSaving(true)
    const id = setTimeout(() => {
      saveTimer.current = null
      let request: Promise<void>
      request = persistNote(activeNote.path, serialized)
        .catch((err: unknown) => {
          console.error('Autosave failed', err)
        })
        .finally(() => {
          if (savePromise.current === request) savePromise.current = null
          setSaving(false)
        })
      savePromise.current = request
    }, AUTOSAVE_MS)
    saveTimer.current = id
    return () => {
      clearTimeout(id)
      if (saveTimer.current === id) saveTimer.current = null
    }
  }, [draft, activeNote, persistNote])

  const loadNote = useCallback(async (rel: string) => {
    // HTML documents render in the preview, not Lexical: no content read,
    // no draft, no autosave — the preview iframe loads via vault:// itself.
    if (isHtmlPath(rel)) {
      setActiveNote({ path: rel, name: rel.split('/').pop() ?? rel, content: '', updatedAt: 0 })
      setDraft(null)
      return
    }
    const note = await window.api.notes.read(rel)
    setActiveNote(note)
    setDraft(parseNoteDocument(note.content))
  }, [])

  // Browser-style history over opened paths: openNote pushes, back/forward
  // move the cursor and load without pushing.
  const nav = useRef({ stack: [] as string[], cursor: -1 })
  const [, setNavTick] = useState(0)

  const pushNav = useCallback((rel: string) => {
    const { stack, cursor } = nav.current
    if (stack[cursor] === rel) return
    nav.current = { stack: [...stack.slice(0, cursor + 1), rel], cursor: cursor + 1 }
    setNavTick((t) => t + 1)
  }, [])

  const openNote = useCallback(
    async (rel: string) => {
      await loadNote(rel)
      pushNav(rel)
    },
    [loadNote, pushNav]
  )

  const goBack = useCallback(async () => {
    const { stack, cursor } = nav.current
    if (cursor <= 0) return
    nav.current = { stack, cursor: cursor - 1 }
    setNavTick((t) => t + 1)
    await loadNote(stack[cursor - 1]).catch(() => {})
  }, [loadNote])

  const goForward = useCallback(async () => {
    const { stack, cursor } = nav.current
    if (cursor >= stack.length - 1) return
    nav.current = { stack, cursor: cursor + 1 }
    setNavTick((t) => t + 1)
    await loadNote(stack[cursor + 1]).catch(() => {})
  }, [loadNote])

  const createNote = useCallback(
    async (dirRel: string, name: string) => {
      const note = await window.api.notes.create(dirRel, name)
      setActiveNote(note)
      setDraft(parseNoteDocument(note.content))
      pushNav(note.path)
      return note
    },
    [pushNav]
  )

  const closeNote = useCallback(() => {
    setActiveNote(null)
    setDraft(null)
  }, [])

  // Clear the open note + history — paths are per-project, so a switch resets them.
  const resetActiveDoc = useCallback(() => {
    setActiveNote(null)
    setDraft(null)
    nav.current = { stack: [], cursor: -1 }
    setNavTick((t) => t + 1)
  }, [])

  const removeEntry = useCallback(
    async (rel: string): Promise<void> => {
      const removesActive =
        activeNote !== null &&
        (activeNote.path === rel || activeNote.path.startsWith(`${rel}/`))

      if (removesActive) {
        if (saveTimer.current) {
          clearTimeout(saveTimer.current)
          saveTimer.current = null
        }
        if (savePromise.current) await savePromise.current
        if (activeNote && draft && !isHtmlPath(activeNote.path)) {
          setSaving(true)
          try {
            await persistNote(activeNote.path, serializeNoteDocument(draft))
          } finally {
            setSaving(false)
          }
        }
      }

      await window.api.entries.remove(rel)
      if (removesActive) resetActiveDoc()
    },
    [activeNote, draft, persistNote, resetActiveDoc]
  )

  const revealEntry = useCallback(async (rel: string): Promise<void> => {
    await window.api.entries.reveal(rel)
  }, [])

  // Open the folder picker, register the chosen folder as a project, switch to it.
  const addProject = useCallback(async () => {
    const res = await window.api.projects.add()
    if (!res) return
    setVaultState(res)
    resetActiveDoc()
    await refreshProjects()
  }, [resetActiveDoc, refreshProjects])

  const switchProject = useCallback(
    async (rootPath: string) => {
      const res = await window.api.projects.switch(rootPath)
      setVaultState(res)
      resetActiveDoc()
      await refreshProjects()
    },
    [resetActiveDoc, refreshProjects]
  )

  const setProjectColor = useCallback(async (rootPath: string, color: string) => {
    const list = await window.api.projects.setColor(rootPath, color)
    setProjects(list)
    const active = list.find((p) => p.active)
    if (active) {
      setVaultState((s) => (s.status === 'ready' ? { ...s, color: active.color } : s))
    }
  }, [])

  // The editor's only write path: replace the body, keep frontmatter verbatim.
  const setDraftBody = useCallback((body: string) => {
    setDraft((prev) => (prev ? { ...prev, body } : prev))
  }, [])

  const setDraftFrontmatter = useCallback((frontmatter: string) => {
    setDraft((prev) => (prev ? { ...prev, frontmatter } : prev))
  }, [])

  return {
    vaultState,
    projects,
    tree,
    gitStatus,
    activeNote,
    draft,
    noteVersion,
    markerPatch,
    saving,
    setDraftBody,
    setDraftFrontmatter,
    addProject,
    switchProject,
    setProjectColor,
    openNote,
    createNote,
    removeEntry,
    revealEntry,
    closeNote,
    goBack,
    goForward,
    canBack: nav.current.cursor > 0,
    canForward: nav.current.cursor < nav.current.stack.length - 1
  }
}
