//! The index — a throwaway view over the source (IN_PRINCIPLES.md: source is
//! truth, index is a cache). Rebuilt from a full scan; never persisted.

use crate::markers::Kind;
use crate::parser::{parse_line, FenceState, LineResult, Marker, Status};
use crate::walk::walk_markdown;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// A marker located in the source. Carries path + line so a face can point at
/// it (IN_PRD.md: each indiana carries path, line, id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Located {
    pub path: PathBuf,
    pub line: usize,
    pub kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

/// The scanned state.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Index {
    pub markers: Vec<Located>,
    pub warnings: Vec<String>,
}

impl Index {
    /// Scan all markdown under `root` into a fresh index.
    pub fn build(root: &Path) -> Index {
        let mut idx = Index::default();
        let mut paths = walk_markdown(root);
        paths.sort();
        for path in paths {
            idx.scan_file(&path);
        }
        idx
    }

    /// Scan one file's markers into the index (used by the walk and, later, by
    /// per-path rescans on watch — IN_SCAN.md: rescan a path, not the repo).
    pub fn scan_file(&mut self, path: &Path) {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                self.warnings.push(format!("{}: unreadable ({e})", path.display()));
                return;
            }
        };
        let mut st = FenceState::default();
        for (i, line) in text.lines().enumerate() {
            let line_no = i + 1;
            match parse_line(line, &mut st) {
                LineResult::Marker(m) => self.markers.push(locate(path, line_no, m)),
                LineResult::Ambiguous => self.warnings.push(format!(
                    "{}:{line_no}: two or more markers on one line — skipped",
                    path.display()
                )),
                LineResult::None => {}
            }
        }
        if st.unclosed_at_eof() {
            self.warnings
                .push(format!("{}: unclosed code fence at EOF", path.display()));
        }
    }
}

fn locate(path: &Path, line: usize, m: Marker) -> Located {
    Located {
        path: path.to_path_buf(),
        line,
        kind: m.kind,
        message: m.message,
        id: m.id,
        status: m.status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Test harness (IN_TEST.md): walk a fixture dir, return its index.
    fn scan_fixture(dir: &Path) -> Index {
        Index::build(dir)
    }

    fn tmp() -> PathBuf {
        let d = std::env::temp_dir().join(format!("indiana-test-{}", nonce()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn nonce() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Atomic counter makes dirs unique even under parallel test threads.
        format!("{nanos}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    fn write(dir: &Path, rel: &str, body: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    #[test]
    fn test_empty_fixture() {
        let d = tmp();
        assert!(scan_fixture(&d).markers.is_empty());
        fs::remove_dir_all(&d).ok();
    }

    // IN_TEST.md E5: full walk of all markdown across subdirs; non-md ignored.
    #[test]
    fn test_full_walk() {
        let d = tmp();
        write(&d, "a.md", "::h\n");
        write(&d, "sub/b.md", "::l\n");
        write(&d, "sub/deep/c.md", "::k\n");
        write(&d, "notes.txt", "::h\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 3);
        fs::remove_dir_all(&d).ok();
    }

    // IN_TEST.md E5: `.indiana/` excluded.
    #[test]
    fn test_exclude_indiana_dir() {
        let d = tmp();
        write(&d, "real.md", "::h\n");
        write(&d, ".indiana/scratch.md", "::l\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 1);
        assert_eq!(idx.markers[0].kind, Kind::Hate);
        fs::remove_dir_all(&d).ok();
    }

    // IN_TEST.md E5: non-markdown files skipped.
    #[test]
    fn test_skip_non_markdown() {
        let d = tmp();
        write(&d, "a.txt", "::h\n");
        write(&d, "b.rs", "::h\n");
        write(&d, "c.json", "::h\n");
        assert!(scan_fixture(&d).markers.is_empty());
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_located_fields() {
        let d = tmp();
        write(&d, "x.md", "intro\n::action[bear-mouse:done] ship it\n");
        let idx = scan_fixture(&d);
        let m = &idx.markers[0];
        assert_eq!(m.line, 2);
        assert_eq!(m.kind, Kind::Action);
        assert_eq!(m.id.as_deref(), Some("bear-mouse"));
        assert_eq!(m.status, Some(Status::Done));
        assert_eq!(m.message.as_deref(), Some("ship it"));
        fs::remove_dir_all(&d).ok();
    }
}
