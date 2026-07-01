import { useCallback, useEffect, useState } from 'react'
import type { Note, TreeNode, VaultState } from '@shared/domain'

const AUTOSAVE_MS = 500

/**
 * Orchestrates vault + tree + the currently open note. All filesystem access
 * goes through `window.api`; the renderer never touches disk directly.
 */
export function useVault() {
  const [vaultState, setVaultState] = useState<VaultState>({ status: 'unset' })
  const [tree, setTree] = useState<TreeNode | null>(null)
  const [activeNote, setActiveNote] = useState<Note | null>(null)
  const [draft, setDraft] = useState<string>('')
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

  // Debounced autosave: when `draft` diverges from the saved note, persist.
  useEffect(() => {
    if (!activeNote || draft === activeNote.content) return
    setSaving(true)
    const id = setTimeout(async () => {
      const saved = await window.api.notes.write(activeNote.path, draft)
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
    const note = await window.api.notes.read(rel)
    setActiveNote(note)
    setDraft(note.content)
  }, [])

  const createNote = useCallback(async (dirRel: string, name: string) => {
    const note = await window.api.notes.create(dirRel, name)
    setActiveNote(note)
    setDraft(note.content)
    return note
  }, [])

  const closeNote = useCallback(() => {
    setActiveNote(null)
    setDraft('')
  }, [])

  return {
    vaultState,
    tree,
    activeNote,
    draft,
    saving,
    setDraft,
    chooseVault,
    openNote,
    createNote,
    closeNote
  }
}
