//! Versioned system prompt prepended to every agent-facing payload.
//!
//! Authoring source: `crates/core/templates/system_prompt.md`.
//! Instance: `.indiana/SYSTEM_PROMPT.md` per monitored root (scaffold on
//! init/refresh; upgrades never overwrite — IN_FOLDER.md).

use serde::Deserialize;
use std::path::{Path, PathBuf};

const EMBEDDED: &str = include_str!("../templates/system_prompt.md");

/// Where a resolved system prompt came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemPromptSource {
    Embedded,
    Root,
}

/// The system prompt value object: body stamped with version, plus resolution metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPrompt {
    /// Renderable text, first line stamped `INDIANA LOOP v{version} — …`.
    pub body: String,
    pub version: u32,
    pub source: SystemPromptSource,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    status: String,
    purpose: String,
    approval: String,
    version: u32,
}

impl SystemPrompt {
    /// Parse the embedded template. Panics on malformed content (compile-time seed).
    pub fn embedded() -> Self {
        parse_template(EMBEDDED, SystemPromptSource::Embedded)
            .unwrap_or_else(|e| panic!("embedded system_prompt.md: {e}"))
    }

    /// Resolve for a monitored root. Valid instance wins; missing → embedded;
    /// invalid → embedded + warning. Outdated instance (version < embedded)
    /// still wins but warns — never auto-overwrite.
    pub fn for_root(root: &Path) -> Self {
        let path = system_prompt_path(root);
        if !path.exists() {
            return Self::embedded();
        }
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                let mut sp = Self::embedded();
                sp.warnings.push(format!(
                    "{}: {e}; falling back to embedded system prompt",
                    path.display()
                ));
                return sp;
            }
        };
        match parse_template(&text, SystemPromptSource::Root) {
            Ok(mut sp) => {
                if let Some(msg) =
                    outdated_warning(sp.version, Self::embedded().version, &path)
                {
                    sp.warnings.push(msg);
                }
                sp
            }
            Err(e) => {
                let mut sp = Self::embedded();
                sp.warnings.push(format!(
                    "{}: {e}; falling back to embedded system prompt",
                    path.display()
                ));
                sp
            }
        }
    }

    /// Parse a system-prompt-shaped file at an arbitrary path (agent personas
    /// share the format — frontmatter with `version`, then the body). The
    /// caller decides the fallback; this only reads and validates.
    pub fn for_path(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        parse_template(&text, SystemPromptSource::Root)
    }

    /// Embedded template bytes for scaffolding into `.indiana/SYSTEM_PROMPT.md`.
    pub fn embedded_raw() -> &'static str {
        EMBEDDED
    }
}

pub fn system_prompt_path(root: &Path) -> PathBuf {
    root.join(".indiana").join("SYSTEM_PROMPT.md")
}

fn parse_template(text: &str, source: SystemPromptSource) -> Result<SystemPrompt, String> {
    let (frontmatter, body) = split_frontmatter(text)?;
    let parsed: Frontmatter = serde_yml::from_str(frontmatter).map_err(|e| e.to_string())?;
    validate_frontmatter(&parsed)?;
    let body = body.trim();
    if body.is_empty() {
        return Err("missing system prompt body".to_string());
    }
    let stamped = stamp_version(body, parsed.version);
    Ok(SystemPrompt {
        body: stamped,
        version: parsed.version,
        source,
        warnings: Vec::new(),
    })
}

fn validate_frontmatter(f: &Frontmatter) -> Result<(), String> {
    for (name, value) in [
        ("status", &f.status),
        ("purpose", &f.purpose),
        ("approval", &f.approval),
    ] {
        if value.trim().is_empty() {
            return Err(format!("frontmatter field `{name}` is empty"));
        }
    }
    if f.version == 0 {
        return Err("frontmatter field `version` must be >= 1".to_string());
    }
    Ok(())
}

fn outdated_warning(instance: u32, embedded: u32, path: &Path) -> Option<String> {
    (instance < embedded).then(|| {
        format!(
            "system prompt outdated (v{instance} < v{embedded}): delete {} and run indiana templates refresh",
            path.display()
        )
    })
}

fn stamp_version(body: &str, version: u32) -> String {
    let mut lines = body.lines();
    let Some(first) = lines.next() else {
        return body.to_string();
    };
    let rest: String = lines.collect::<Vec<_>>().join("\n");
    let after = match first.strip_prefix("INDIANA LOOP") {
        Some(tail) => {
            let tail = tail.trim_start();
            let tail = match tail.strip_prefix('v') {
                Some(vrest) => vrest
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start(),
                None => tail,
            };
            tail.strip_prefix('—').map(str::trim_start).unwrap_or(tail)
        }
        None => first,
    };
    let stamped_first = if after.is_empty() {
        format!("INDIANA LOOP v{version}")
    } else {
        format!("INDIANA LOOP v{version} — {after}")
    };
    if rest.is_empty() {
        stamped_first
    } else {
        format!("{stamped_first}\n{rest}")
    }
}

fn split_frontmatter(text: &str) -> Result<(&str, &str), String> {
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| "missing YAML frontmatter".to_string())?;
    let Some(end) = rest.find("\n---\n") else {
        return Err("unclosed YAML frontmatter".to_string());
    };
    Ok((&rest[..end], &rest[end + "\n---\n".len()..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "indiana-system-prompt-{n}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_instance(root: &Path, version: u32, body: &str) {
        let path = system_prompt_path(root);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "---\nstatus: draft\npurpose: test\napproval: pending\nversion: {version}\n---\n\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_embedded_system_prompt_parses_and_has_version() {
        let sp = SystemPrompt::embedded();
        assert!(sp.version >= 1);
        assert_eq!(sp.source, SystemPromptSource::Embedded);
        assert!(sp.warnings.is_empty());
        assert!(
            sp.body.starts_with(&format!("INDIANA LOOP v{} —", sp.version)),
            "stamped version: {}",
            sp.body.lines().next().unwrap_or("")
        );
        assert!(sp.body.contains(".indiana/context-model/CONTEXT-MODEL.md"));
        assert!(sp.body.contains(".indiana/chief-of-staff/focus.md"));
        assert!(sp.body.contains("one commit per command"));
        assert!(sp.body.contains("never push"));
    }

    #[test]
    fn test_system_prompt_names_fundamentals() {
        // Pinned here — packaged crate builds carry only the crate dir, so we
        // do not read FUNDAMENTALS.md from the repo root.
        let required = [
            "SINGLE SOURCE OF TRUTH",
            "ELEPHANT PRINCIPLE",
            "CONTENT OVER CHAT",
            "FOLDER IS THE UNIT OF WORK",
            "KNOWLEDGE COMPOUNDS THROUGH LOOPS",
            "DOMAIN ARCHITECTURE > TECH",
            "HARNESS AGNOSTIC",
            "CONE-SHAPED TREE",
            "FILE LIFE CYCLE",
            "DEPENDENCY MANAGEMENT",
            "FRONTMATTER ON EVERY FILE",
            "DOCUMENT WHY, NOT WHAT",
            "PROMOTE, NEVER FORK",
            "MARKDOWN AS CODE",
        ];
        let body = SystemPrompt::embedded().body;
        for name in required {
            assert!(
                body.contains(name),
                "embedded system prompt must name {name}"
            );
        }
    }

    #[test]
    fn test_root_system_prompt_overrides_embedded() {
        let d = tmp();
        write_instance(
            &d,
            1,
            "INDIANA LOOP — custom root prompt. Read nothing else.",
        );
        let sp = SystemPrompt::for_root(&d);
        assert_eq!(sp.source, SystemPromptSource::Root);
        assert!(sp.body.contains("custom root prompt"));
        assert!(sp.body.starts_with("INDIANA LOOP v1 —"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_bad_system_prompt_falls_back_with_warning() {
        let d = tmp();
        let path = system_prompt_path(&d);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "not a valid system prompt\n").unwrap();
        let sp = SystemPrompt::for_root(&d);
        assert_eq!(sp.source, SystemPromptSource::Embedded);
        assert_eq!(sp.warnings.len(), 1);
        assert!(sp.warnings[0].contains("falling back"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_outdated_system_prompt_warns() {
        let d = tmp();
        let path = system_prompt_path(&d);
        let msg = outdated_warning(1, 2, &path).expect("1 < 2 is outdated");
        assert!(msg.contains("outdated (v1 < v2)"));
        assert!(msg.contains("templates refresh"));
        assert!(outdated_warning(2, 2, &path).is_none());
        assert!(outdated_warning(3, 2, &path).is_none());

        // When embedded advances past an instance, for_root surfaces the warning
        // without overwriting the file.
        let embedded = SystemPrompt::embedded();
        if embedded.version > 1 {
            write_instance(&d, 1, "INDIANA LOOP — old.");
            let sp = SystemPrompt::for_root(&d);
            assert_eq!(sp.source, SystemPromptSource::Root);
            assert!(sp.warnings.iter().any(|w| w.contains("outdated")));
        } else {
            write_instance(&d, embedded.version, "INDIANA LOOP — current.");
            let sp = SystemPrompt::for_root(&d);
            assert!(
                !sp.warnings.iter().any(|w| w.contains("outdated")),
                "same version must not warn: {:?}",
                sp.warnings
            );
        }
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_missing_system_prompt_uses_embedded() {
        let d = tmp();
        let sp = SystemPrompt::for_root(&d);
        assert_eq!(sp.source, SystemPromptSource::Embedded);
        assert!(sp.warnings.is_empty());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_stamp_version_rewrites_first_line() {
        assert_eq!(
            stamp_version("INDIANA LOOP — hello", 3),
            "INDIANA LOOP v3 — hello"
        );
        assert_eq!(
            stamp_version("INDIANA LOOP v1 — hello", 2),
            "INDIANA LOOP v2 — hello"
        );
    }
}
