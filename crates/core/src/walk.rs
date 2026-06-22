//! Walk markdown under a root, fast (IN_SCAN.md: full walk on startup).
//! Read-only. Excludes `.indiana/` (Indiana's own scratch is not content)
//! and `.git/` (VCS internals, never review content).

use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Directory names pruned from the walk.
const PRUNE: &[&str] = &[".indiana", ".git"];

/// Every `.md` file under `root`, pruned dirs excluded. Order is unspecified.
pub fn walk_markdown(root: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        // Walk all markdown, not just what VCS would show (IN_SCAN.md).
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .ignore(false)
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
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect()
}
