// ::ignore
//! The index — a throwaway view over the source (IN_PRINCIPLES.md: source is
//! truth, index is a cache). Rebuilt from a full scan; never persisted.

use crate::agents::AgentCatalog;
use crate::markers::Kind;
use crate::parser::{file_ignored, parse_line_with, FenceState, LineResult, Marker, Status};
use crate::scope::{self, Scope};
use crate::walk::walk_files;
use crate::write::{self, InjectRequest, WriteResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A marker located in the source. Carries path + line so a face can point at
/// it (IN_PRD.md: each indiana carries path, line, id).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Located {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub kind: Kind,
    pub raw_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Numeric manual batch label (`-1`, `-2`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<u64>,
    /// Named agent persona (`-m` / `-mike`); canonical agent name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    /// The `-a` / `--auto` flag was present (IN_AUTORUN.md). The daemon reads
    /// this to decide dispatch; faces ignore it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub auto: bool,
    pub scope: Scope,
}

/// The scanned state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Index {
    pub markers: Vec<Located>,
    pub warnings: Vec<String>,
}

/// Per-kind tallies (IN_PRD.md: copy and counts). Computed by the core; a face
/// only displays this, never counts itself (IN_PRINCIPLES.md: faces render).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counts {
    pub question: usize,
    pub hate: usize,
    pub love: usize,
    pub keep: usize,
    pub fix: usize,
    pub elaborate: usize,
    pub note: usize,
    pub action: usize,
    pub todo: usize,
    pub delete: usize,
    pub prompt: usize,
}

impl Counts {
    pub fn total(&self) -> usize {
        self.question
            + self.hate
            + self.love
            + self.keep
            + self.fix
            + self.elaborate
            + self.note
            + self.action
            + self.todo
            + self.delete
            + self.prompt
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScanOptions {
    pub read_only: bool,
    /// Run the chief-of-staff capture/reconcile pass (COS_PRD.md). Off by
    /// default: only deliberate entry points (daemon rebuilds, explicit
    /// `indiana scan <path>`) may mint `.indiana/chief-of-staff/` files —
    /// a plain write scan injects ids and nothing else.
    pub capture: bool,
}

impl ScanOptions {
    pub fn write_ids() -> Self {
        Self {
            read_only: false,
            capture: false,
        }
    }

    pub fn with_capture(mut self) -> Self {
        self.capture = true;
        self
    }

    pub fn read_only() -> Self {
        Self {
            read_only: true,
            capture: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    pub index: Index,
    pub written_paths: Vec<PathBuf>,
}

impl Index {
    /// Build an index and perform the M7 tracked-id injection pass.
    pub fn build(root: &Path) -> Self {
        Self::build_with_options(root, ScanOptions::write_ids()).index
    }

    /// Build an index without mutating source markdown.
    pub fn build_read_only(root: &Path) -> Self {
        Self::build_with_options(root, ScanOptions::read_only()).index
    }

    /// Build an index, optionally writing IDs into tracked marker lines.
    pub fn build_with_options(root: &Path, options: ScanOptions) -> ScanReport {
        // Agent flags (`-m`, `-mike`) only resolve against the root's defined
        // agents, so the catalog is loaded once per build.
        let agents = AgentCatalog::for_root(root);
        let mut first = Index::default();
        for path in walk_files(root) {
            first.scan_file_with(&path, &agents);
        }

        let requests: Vec<InjectRequest> = first
            .markers
            .iter()
            .filter(|m| crate::markers::is_tracked(m.kind))
            .map(|m| InjectRequest {
                path: m.path.clone(),
                line_no: m.line,
            })
            .collect();

        let results = write::inject(&requests, options.read_only);
        let written_paths: Vec<PathBuf> = results
            .iter()
            .filter_map(|(path, result)| {
                if *result == WriteResult::Written {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();

        let index = if options.read_only || written_paths.is_empty() {
            first
        } else {
            let mut second = Index::default();
            for path in walk_files(root) {
                second.scan_file_with(&path, &agents);
            }
            second
        };

        let mut written_paths = written_paths;
        if !options.read_only && options.capture {
            // Chief-of-staff capture/reconcile (COS_PRD.md): tracked ids become
            // tasks.md rows; tracker writes join OwnWriteTracker via written_paths.
            let report = crate::cos::capture_and_reconcile(root, &index);
            written_paths.extend(report.written);
        }

        ScanReport {
            index,
            written_paths,
        }
    }

    /// Per-kind tallies over the current markers.
    pub fn counts(&self) -> Counts {
        let mut c = Counts::default();
        for m in &self.markers {
            match m.kind {
                Kind::Question => c.question += 1,
                Kind::Hate => c.hate += 1,
                Kind::Love => c.love += 1,
                Kind::Keep => c.keep += 1,
                Kind::Fix => c.fix += 1,
                Kind::Elaborate => c.elaborate += 1,
                Kind::Note => c.note += 1,
                Kind::Action => c.action += 1,
                Kind::Todo => c.todo += 1,
                Kind::Delete => c.delete += 1,
                Kind::Prompt => c.prompt += 1,
            }
        }
        c
    }

    /// Scan one file with no known agents (see `scan_file_with`).
    pub fn scan_file(&mut self, path: &Path) {
        self.scan_file_with(path, &AgentCatalog::default())
    }

    /// Scan one file's markers into the index (used by the walk and, later, by
    /// per-path rescans on watch — IN_SCAN.md: rescan a path, not the repo).
    /// `agents` supplies the known persona flag tokens for the owning root.
    pub fn scan_file_with(&mut self, path: &Path, agents: &AgentCatalog) {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            // Not UTF-8 → binary, silently out of scope (IN_SCAN.md: the walk
            // visits every file; text detection happens here, at read time).
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => return,
            Err(e) => {
                self.warnings
                    .push(format!("{}: unreadable ({e})", path.display()));
                return;
            }
        };
        // File-level opt-out (IN_SCAN.md): `::ignore` in frontmatter or as a
        // first-line comment silences the whole file — no markers, no warnings.
        if file_ignored(&text) {
            return;
        }
        let mut st = FenceState::default();
        let start = self.markers.len();
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let line_no = i + 1;
            match parse_line_with(line, &mut st, agents) {
                LineResult::Marker(m) => self.markers.push(locate(path, line_no, m)),
                LineResult::Ambiguous => self.warnings.push(format!(
                    "{}:{line_no}: two or more markers on one line — skipped",
                    path.display()
                )),
                LineResult::None => {}
            }
        }
        for marker in &mut self.markers[start..] {
            marker.scope = scope::resolve(&lines, marker.line, marker.column);
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
        raw_token: m.raw_token,
        message: m.message,
        group: m.group,
        agent: m.agent,
        id: m.id,
        status: m.status,
        column: m.column,
        auto: m.auto,
        scope: Scope {
            kind: scope::ScopeKind::NextRow,
            content: String::new(),
        },
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

    // IN_TEST.md E5: full walk of all files across subdirs and extensions.
    #[test]
    fn test_full_walk() {
        let d = tmp();
        write(&d, "a.md", "::h\n");
        write(&d, "sub/b.md", "::l\n");
        write(&d, "sub/deep/c.md", "::k\n");
        write(&d, "notes.txt", "::h\n");
        write(&d, "Makefile", "all: ## ::k\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 5);
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

    // IN_TEST.md E5: code files are scanned; markers ride comments.
    #[test]
    fn test_scan_code_files() {
        let d = tmp();
        write(&d, "main.rs", "fn main() {} // ::fix rename this\n");
        write(&d, "app.py", "import os  # ::q why os here\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 2);
        assert_eq!(idx.markers.iter().filter(|m| m.kind == Kind::Fix).count(), 1);
        assert_eq!(
            idx.markers.iter().filter(|m| m.kind == Kind::Question).count(),
            1
        );
        fs::remove_dir_all(&d).ok();
    }

    // IN_SCAN.md: glued `::` is a path separator, never a marker.
    #[test]
    fn test_code_paths_are_not_markers() {
        let d = tmp();
        write(
            &d,
            "lib.rs",
            "use std::fs;\nlet k = Kind::Action;\nlet x = path::f(y);\nstd::f32::MAX;\n",
        );
        let idx = scan_fixture(&d);
        assert!(idx.markers.is_empty(), "got: {:?}", idx.markers);
        fs::remove_dir_all(&d).ok();
    }

    // IN_SCAN.md E5: binary files are silently out of scope.
    #[test]
    fn test_skip_binary_files() {
        let d = tmp();
        fs::write(d.join("blob.bin"), [0xFFu8, 0xFE, 0x00, b':', b':', b'h']).unwrap();
        let idx = scan_fixture(&d);
        assert!(idx.markers.is_empty());
        assert!(idx.warnings.is_empty(), "binary skip must not warn");
        fs::remove_dir_all(&d).ok();
    }

    // IN_SCAN.md E5: gitignored paths are excluded from the walk.
    #[test]
    fn test_gitignored_paths_excluded() {
        let d = tmp();
        write(&d, ".gitignore", "dist/\n*.log\n");
        write(&d, "kept.md", "::h\n");
        write(&d, "dist/out.js", "// ::fix generated\n");
        write(&d, "run.log", "boot ::l\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 1);
        assert_eq!(idx.markers[0].kind, Kind::Hate);
        fs::remove_dir_all(&d).ok();
    }

    // IN_SCAN.md E5: node_modules / target pruned even without a .gitignore.
    #[test]
    fn test_prune_dep_dirs_without_gitignore() {
        let d = tmp();
        write(&d, "kept.md", "::h\n");
        write(&d, "node_modules/pkg/index.js", "// ::fix vendored\n");
        write(&d, "target/debug/out.d", "x ::l\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 1);
        fs::remove_dir_all(&d).ok();
    }

    // IN_LINE.md: tracked markers in code files get IDs like markdown ones.
    #[test]
    fn test_id_injection_in_code_file() {
        let d = tmp();
        write(&d, "main.rs", "fn main() {} // ::todo wire the flag\n");
        let idx = scan_fixture(&d);
        assert!(idx.markers[0].id.is_some());
        let text = fs::read_to_string(d.join("main.rs")).unwrap();
        assert!(text.contains("// ::todo["), "got: {text:?}");
        fs::remove_dir_all(&d).ok();
    }

    // IN_SCAN.md: `::ignore` in frontmatter silences the whole file.
    #[test]
    fn test_ignored_file_contributes_nothing() {
        let d = tmp();
        write(&d, "kept.md", "::h\n");
        write(
            &d,
            "ignored.md",
            "---\nstatus: draft\n# ::ignore\n---\n::l\n::action do it\n```\nunclosed fence\n",
        );
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 1);
        assert_eq!(idx.markers[0].kind, Kind::Hate);
        // No warnings either — the file is out of scope entirely.
        assert!(idx.warnings.is_empty());
        // No ID injection into the ignored file.
        assert!(!fs::read_to_string(d.join("ignored.md"))
            .unwrap()
            .contains("::action["));
        fs::remove_dir_all(&d).ok();
    }

    // IN_PRD.md / E9: core computes per-kind tallies; faces just read them.
    #[test]
    fn test_counts() {
        let d = tmp();
        write(&d, "m.md", "::h\n::h\n::l\n::fix go\n::action[x] do\n");
        let c = scan_fixture(&d).counts();
        assert_eq!(c.hate, 2);
        assert_eq!(c.love, 1);
        assert_eq!(c.fix, 1);
        assert_eq!(c.action, 1);
        assert_eq!(c.total(), 5);
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

    #[test]
    fn test_index_carries_numeric_group() {
        let d = tmp();
        write(&d, "x.md", "::fix -12 ship it\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers[0].group, Some(12));
        assert_eq!(idx.markers[0].message.as_deref(), Some("ship it"));
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_index_carries_agent_when_defined() {
        let d = tmp();
        crate::agents::scaffold_agents(&d).unwrap();
        write(&d, "x.md", "::fix -m create this task\n::fix -lisa shape it\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers[0].agent.as_deref(), Some("mike"));
        assert_eq!(idx.markers[0].message.as_deref(), Some("create this task"));
        assert_eq!(idx.markers[1].agent.as_deref(), Some("lisa"));
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_index_agent_flag_without_agents_stays_message() {
        let d = tmp();
        write(&d, "x.md", "::fix -m create this task\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers[0].agent, None);
        assert_eq!(idx.markers[0].message.as_deref(), Some("-m create this task"));
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_index_injects_tracked_ids() {
        let d = tmp();
        write(&d, "x.md", "::action ship it\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers[0].kind, Kind::Action);
        assert!(idx.markers[0].id.is_some());
        assert!(fs::read_to_string(d.join("x.md"))
            .unwrap()
            .starts_with("::action["));
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_index_only_tracked_get_ids() {
        let d = tmp();
        write(&d, "x.md", "::h\n::todo do it\n");
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 2);
        assert_eq!(idx.markers[0].id, None);
        assert!(idx.markers[1].id.is_some());
        let text = fs::read_to_string(d.join("x.md")).unwrap();
        assert!(text.starts_with("::h\n::todo["));
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_index_read_only_no_write() {
        let d = tmp();
        write(&d, "x.md", "::action ship it\n");
        let before = fs::read(d.join("x.md")).unwrap();
        let idx = Index::build_read_only(&d);
        assert_eq!(idx.markers[0].id, None);
        assert_eq!(before, fs::read(d.join("x.md")).unwrap());
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_index_resolves_scope() {
        let d = tmp();
        write(
            &d,
            "x.md",
            "Fix this ::f\n::n note\nnext line\n\n::k\n## Head\nbody\n## Next\n",
        );
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers[0].scope.content, "Fix this");
        assert_eq!(idx.markers[1].scope.content, "next line");
        assert_eq!(idx.markers[2].scope.content, "## Head\nbody");
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_frontmatter_property_comment_scope_and_identity() {
        let d = tmp();
        write(
            &d,
            "x.md",
            "---\nstatus: draft\n# frontmatter.status ::todo approve it\n---\n\nBody.\n",
        );
        let idx = scan_fixture(&d);
        assert_eq!(idx.markers.len(), 1);
        assert_eq!(idx.markers[0].scope.content, "# frontmatter.status");
        assert!(idx.markers[0].id.is_some());
        assert!(fs::read_to_string(d.join("x.md"))
            .unwrap()
            .contains("# frontmatter.status ::todo["));
        fs::remove_dir_all(&d).ok();
    }
}
