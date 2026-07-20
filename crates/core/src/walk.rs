//! Walk every file under a root, fast (IN_SCAN.md: full walk on startup).
//! Read-only. Excludes `.indiana/` (Indiana's own scratch is not content)
//! and `.git/` (VCS internals, never review content). Honors `.gitignore`:
//! ignored paths are build artifacts and dependency trees, not content —
//! untracked files still walk (untracked ≠ ignored).

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Directory names pruned from the walk regardless of `.gitignore` —
/// a repo without one must still never scan its build/dep trees.
const PRUNE: &[&str] = &[".indiana", ".git", "node_modules", "target"];

/// Every file under `root`, pruned and gitignored dirs excluded. Order is
/// unspecified. Binary detection happens at read time (invalid UTF-8 is
/// skipped by the scanner), not here.
pub fn walk_files(root: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true)
        .ignore(true)
        // Honor `.gitignore` even when the root is not a git repo (monitored
        // folders and test fixtures need not be repos).
        .require_git(false)
        .hidden(false)
        .parents(false)
        .filter_entry(|e| {
            !e.file_name()
                .to_str()
                .map(|n| PRUNE.contains(&n))
                .unwrap_or(false)
        })
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.into_path())
        .collect()
}
