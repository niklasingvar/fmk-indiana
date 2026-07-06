import { useCallback, useEffect, useState } from 'react'
import type { GitStatusMap, Note, NoteDocument, TreeNode, VaultState } from '@shared/domain'
import { isHtmlPath } from '@shared/annotation-line'
import { parseNoteDocument, serializeNoteDocument } from '@shared/note-serialization'

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
  const [tree, setTree] = useState<TreeNode | null>(null)
  const [gitStatus, setGitStatus] = useState<GitStatusMap>({})
  const [activeNote, setActiveNote] = useState<Note | null>(null)
  const [draft, setDraft] = useState<NoteDocument | null>(null)
  const [saving, setSaving] = useState(false)

  // Load persisted vault on mount.
  useEffect(() => {
    void window.api.vault.get().then((s) => setVaultState(s))
  }, [])

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

  // Debounced autosave: when the serialized draft diverges from the saved
  // note, persist.
  useEffect(() => {
    if (!activeNote || !draft) return
    const serialized = serializeNoteDocument(draft)
    if (serialized === activeNote.content) return
    setSaving(true)
    const id = setTimeout(async () => {
      const saved = await window.api.notes.write(activeNote.path, serialized)
      // Preserve the user's cursor by keeping draft authoritative unless the
      // external content changed something we don't have locally.
      setActiveNote((prev) => (prev ? { ...prev, content: saved.content, updatedAt: saved.updatedAt } : prev))
      setSaving(false)
    }, AUTOSAVE_MS)
    return () => clearTimeout(id)
  }, [draft, activeNote])

  const chooseVault = useCallback(async () => {
    const res = await window.api.vault.choose()
    if (res) setVaultState(res)
  }, [])

  const openNote = useCallback(async (rel: string) => {
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

  const createNote = useCallback(async (dirRel: string, name: string) => {
    const note = await window.api.notes.create(dirRel, name)
    setActiveNote(note)
    setDraft(parseNoteDocument(note.content))
    return note
  }, [])

  const closeNote = useCallback(() => {
    setActiveNote(null)
    setDraft(null)
  }, [])

  // The editor's only write path: replace the body, keep frontmatter verbatim.
  const setDraftBody = useCallback((body: string) => {
    setDraft((prev) => (prev ? { ...prev, body } : prev))
  }, [])

  return {
    vaultState,
    tree,
    gitStatus,
    activeNote,
    draft,
    saving,
    setDraftBody,
    chooseVault,
    openNote,
    createNote,
    closeNote
  }
}
