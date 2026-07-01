//! Cursor store — the set of marker identities already copied.
//! Lives in `~/.indiana/copied.json`. Interaction history, not a cache of source
//! (IN_PRINCIPLES.md carve-out). Safe to delete — `--latest` falls back to copy-all.

use crate::paths::indiana_dir;
use std::collections::HashSet;
use std::io;

/// Load the copied-identity set. Returns empty set on first run or corrupt file.
pub fn load() -> HashSet<String> {
    let path = indiana_dir().join("copied.json");
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .map(|v| v.into_iter().collect())
        .unwrap_or_default()
}

/// Add the just-copied identities to the stored set (union, append-only).
/// No GC by current scan: the cursor is one global file with path-qualified
/// identities, but a copy may scan only a subfolder (or a different root than a
/// previous copy). Intersecting with that scan would drop identities for every
/// file outside it — silent data loss across roots. Append-only is correct for
/// "not yet copied"; growth is bounded by distinct markers ever copied, and the
/// file is safe to delete (falls back to copy-all).
pub fn save(added: &HashSet<String>) -> io::Result<()> {
    let mut set = load();
    set.extend(added.iter().cloned());
    let path = indiana_dir().join("copied.json");
    std::fs::create_dir_all(indiana_dir())?;
    let v: Vec<&String> = set.iter().collect();
    std::fs::write(&path, serde_json::to_string_pretty(&v)?)
}
