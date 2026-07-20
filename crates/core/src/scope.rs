// ::ignore
//! Scope resolution — what a marker carries into the payload.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeKind {
    Inline,
    NextRow,
    Section,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    pub kind: ScopeKind,
    pub content: String,
}

/// Resolve one marker's scope from its file lines. `line_no` is 1-based;
/// `column` is the byte offset of the marker's `::` on that line.
pub fn resolve(lines: &[&str], line_no: usize, column: usize) -> Scope {
    let idx = line_no.saturating_sub(1);
    let line = lines.get(idx).copied().unwrap_or_default();

    if line[..column.min(line.len())].trim().is_empty() {
        if let Some(heading_idx) = next_nonblank(lines, idx + 1) {
            if let Some(level) = atx_heading_level(lines[heading_idx]) {
                return Scope {
                    kind: ScopeKind::Section,
                    content: section(lines, heading_idx, level),
                };
            }
        }

        return Scope {
            kind: ScopeKind::NextRow,
            content: next_block(lines, idx + 1),
        };
    }

    Scope {
        kind: ScopeKind::Inline,
        content: line[..column.min(line.len())].trim_end().to_string(),
    }
}

fn next_nonblank(lines: &[&str], from: usize) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(from)
        .find_map(|(i, line)| (!line.trim().is_empty()).then_some(i))
}

fn next_block(lines: &[&str], from: usize) -> String {
    let Some(start) = next_nonblank(lines, from) else {
        return String::new();
    };
    let end = lines
        .iter()
        .enumerate()
        .skip(start)
        .find_map(|(i, line)| line.trim().is_empty().then_some(i))
        .unwrap_or(lines.len());
    lines[start..end].join("\n")
}

fn section(lines: &[&str], heading_idx: usize, level: usize) -> String {
    let end = lines
        .iter()
        .enumerate()
        .skip(heading_idx + 1)
        .find_map(|(i, line)| {
            atx_heading_level(line)
                .filter(|next_level| *next_level <= level)
                .map(|_| i)
        })
        .unwrap_or(lines.len());
    lines[heading_idx..end].join("\n")
}

fn atx_heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let level = trimmed.bytes().take_while(|b| *b == b'#').count();
    if (1..=6).contains(&level) && trimmed.as_bytes().get(level) == Some(&b' ') {
        Some(level)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_inline() {
        let lines = ["Fix this ::f"];
        let scope = resolve(&lines, 1, 9);
        assert_eq!(scope.kind, ScopeKind::Inline);
        assert_eq!(scope.content, "Fix this");
    }

    #[test]
    fn test_scope_next_row() {
        let lines = ["::n note", "- one", "- two", "", "after"];
        let scope = resolve(&lines, 1, 0);
        assert_eq!(scope.kind, ScopeKind::NextRow);
        assert_eq!(scope.content, "- one\n- two");
    }

    #[test]
    fn test_scope_file_bound() {
        let lines = ["::n note"];
        let scope = resolve(&lines, 1, 0);
        assert_eq!(scope.kind, ScopeKind::NextRow);
        assert_eq!(scope.content, "");
    }

    #[test]
    fn test_scope_section() {
        let lines = ["::k", "## Intro", "text", "### Detail", "more", "## Next"];
        let scope = resolve(&lines, 1, 0);
        assert_eq!(scope.kind, ScopeKind::Section);
        assert_eq!(scope.content, "## Intro\ntext\n### Detail\nmore");
    }

    #[test]
    fn test_scope_nested_section() {
        let lines = ["::k", "### Detail", "more", "### Next", "after"];
        let scope = resolve(&lines, 1, 0);
        assert_eq!(scope.kind, ScopeKind::Section);
        assert_eq!(scope.content, "### Detail\nmore");
    }

    #[test]
    fn test_scope_most_specific() {
        let lines = ["::k", "## Intro", "inner ::f", "## Next"];
        let section_scope = resolve(&lines, 1, 0);
        let inline_scope = resolve(&lines, 3, 6);
        assert_eq!(section_scope.kind, ScopeKind::Section);
        assert_eq!(inline_scope.kind, ScopeKind::Inline);
        assert_eq!(inline_scope.content, "inner");
    }

    #[test]
    fn test_inline_heading_stays_inline() {
        let lines = ["## Intro ::k", "body"];
        let scope = resolve(&lines, 1, 9);
        assert_eq!(scope.kind, ScopeKind::Inline);
        assert_eq!(scope.content, "## Intro");
    }
}
