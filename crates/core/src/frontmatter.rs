//! Default-frontmatter linter. Indiana markdown files open with a standard
//! frontmatter (`status`, `purpose`, `approval`). This module finds `.md`
//! files that lack it and, in `--write` mode, prepends the default block via
//! the shared atomic-write primitive ([`crate::write::atomic_write`]).
//!
//! Check mode is the default and non-mutating; `--write` mutates. The default
//! block is read from `<root>/.indiana/FRONTMATTER.md` (the authoring source),
//! falling back to an embedded default when that file is absent or unparseable.

use crate::write;
use ignore::WalkBuilder;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Embedded default frontmatter block (trailing newline included). Used when a
/// root has no `.indiana/FRONTMATTER.md` or that file has no parseable block.
pub const DEFAULT_FRONTMATTER: &str = "---\nstatus: draft\npurpose: TODO\napproval: pending\n---\n";

/// Directory names pruned from the lint walk. `.indiana`/`.git` are pruned
/// everywhere; `target`/`node_modules` are build/dep trees; `skills` holds
/// third-party skill content that has its own frontmatter convention.
const PRUNE: &[&str] = &[".indiana", ".git", "target", "node_modules", "skills"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontmatterReport {
    /// Files that lack frontmatter (in walk order).
    pub missing: Vec<PathBuf>,
    /// Files that had the default block prepended (only in `--write` mode).
    pub written: Vec<PathBuf>,
}

/// The frontmatter block to prepend for `root`. Reads
/// `<root>/.indiana/FRONTMATTER.md` and extracts its first `--- … ---` block;
/// falls back to [`DEFAULT_FRONTMATTER`] when the file is absent or unparseable.
pub fn default_block(root: &Path) -> String {
    let path = root.join(".indiana").join("FRONTMATTER.md");
    match fs::read_to_string(&path) {
        Ok(text) => match extract_block(&text) {
            Some(block) => block,
            None => DEFAULT_FRONTMATTER.to_string(),
        },
        Err(_) => DEFAULT_FRONTMATTER.to_string(),
    }
}

/// Extract the first `---\n…\n---` block from `text`, including fences and a
/// trailing newline. Returns `None` if no closed block is present.
fn extract_block(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| l.trim() == "---")?;
    let end = (start + 1..lines.len()).find(|&i| lines[i].trim() == "---")?;
    let mut block = lines[start..=end].join("\n");
    block.push('\n');
    Some(block)
}

/// True if `text` already opens with a YAML frontmatter fence.
pub fn has_frontmatter(text: &str) -> bool {
    text.starts_with("---\n") || text.starts_with("---\r\n")
}

/// Walk `root`'s markdown and report files missing frontmatter. With `write`,
/// prepend the default block to each missing file (atomic, mtime-guarded).
pub fn lint(root: &Path, write: bool) -> io::Result<FrontmatterReport> {
    let block = default_block(root);
    let mut report = FrontmatterReport::default();
    for path in walk_markdown_lint(root) {
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        if has_frontmatter(&text) {
            continue;
        }
        report.missing.push(path.clone());
        if write && prepend(&path, &block)? {
            report.written.push(path);
        }
    }
    Ok(report)
}

/// Prepend `block` to `path` unless it gained frontmatter or changed under us.
/// Returns `true` when the file was written.
fn prepend(path: &Path, block: &str) -> io::Result<bool> {
    let before = match fs::metadata(path) {
        Ok(m) => m.modified().ok(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let text = fs::read_to_string(path)?;
    if has_frontmatter(&text) {
        return Ok(false);
    }
    if fs::metadata(path)?.modified().ok() != before {
        return Ok(false);
    }
    // block ends with a fenced newline; a blank line separates it from content.
    let out = format!("{block}\n{text}");
    write::atomic_write(path, out.as_bytes())?;
    Ok(true)
}

fn walk_markdown_lint(root: &Path) -> Vec<PathBuf> {
    WalkBuilder::new(root)
        .git_ignore(true)
        .ignore(true)
        .hidden(true)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "indiana-frontmatter-{nanos}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_file(dir: &Path, rel: &str, body: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    #[test]
    fn test_has_frontmatter() {
        assert!(has_frontmatter("---\nstatus: draft\n---\nbody\n"));
        assert!(!has_frontmatter("# Title\n\nbody\n"));
        assert!(!has_frontmatter(""));
    }

    #[test]
    fn test_default_block_falls_back_when_absent() {
        let d = tmp();
        assert_eq!(default_block(&d), DEFAULT_FRONTMATTER);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_default_block_reads_indiana_file() {
        let d = tmp();
        write_file(
            &d,
            ".indiana/FRONTMATTER.md",
            "# Default\n\n```yaml\n---\nstatus: draft\npurpose: custom\napproval: pending\n---\n```\n",
        );
        assert_eq!(
            default_block(&d),
            "---\nstatus: draft\npurpose: custom\napproval: pending\n---\n"
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_default_block_unparseable_falls_back() {
        let d = tmp();
        write_file(&d, ".indiana/FRONTMATTER.md", "no fences here\n");
        assert_eq!(default_block(&d), DEFAULT_FRONTMATTER);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_lint_check_reports_missing() {
        let d = tmp();
        write_file(&d, "a.md", "# A\n");
        write_file(&d, "b.md", "---\nstatus: draft\n---\n\nB\n");
        let report = lint(&d, false).unwrap();
        assert_eq!(report.missing, vec![d.join("a.md")]);
        assert!(report.written.is_empty());
        // check mode does not mutate
        assert_eq!(fs::read_to_string(d.join("a.md")).unwrap(), "# A\n");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_lint_write_prepends_and_is_idempotent() {
        let d = tmp();
        write_file(&d, "a.md", "# A\nbody\n");
        let report = lint(&d, true).unwrap();
        assert_eq!(report.written, vec![d.join("a.md")]);
        let text = fs::read_to_string(d.join("a.md")).unwrap();
        assert!(text.starts_with(
            "---\nstatus: draft\npurpose: TODO\napproval: pending\n---\n\n# A\nbody\n"
        ));
        // second run: a.md now has frontmatter → not missing, not rewritten
        let report2 = lint(&d, true).unwrap();
        assert!(report2.missing.is_empty());
        assert!(report2.written.is_empty());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_lint_skips_excluded_dirs() {
        let d = tmp();
        write_file(&d, "docs/a.md", "# A\n");
        write_file(&d, "skills/s.md", "# skill\n");
        write_file(&d, "target/b.md", "# build\n");
        let report = lint(&d, false).unwrap();
        assert_eq!(report.missing, vec![d.join("docs/a.md")]);
        fs::remove_dir_all(d).ok();
    }
}
