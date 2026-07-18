//! Named agent personas (Mike, Lisa, …). An agent is a text file:
//! `.indiana/agents/<name>/SYSTEM_PROMPT.md`, a full standalone replacement
//! for the default system prompt. Markers tag an agent with `-<name>` (or the
//! unique first letter, e.g. `-m` for mike); copy/dispatch of that batch
//! prepends the agent's prompt instead of `.indiana/SYSTEM_PROMPT.md`.
//!
//! Authoring sources live in `crates/core/templates/agents/`; init/refresh
//! scaffolds them into a monitored root and never overwrites user edits.

use crate::system_prompt::SystemPrompt;
use std::io;
use std::path::{Path, PathBuf};

/// Default agents every monitored root starts with: (name, embedded template).
pub const EMBEDDED_AGENTS: &[(&str, &str)] = &[
    ("lisa", include_str!("../templates/agents/lisa/system_prompt.md")),
    ("mike", include_str!("../templates/agents/mike/system_prompt.md")),
];

/// The agents defined in one root, discovered from disk. Names are the
/// directory names under `.indiana/agents/` that carry a `SYSTEM_PROMPT.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentCatalog {
    /// Sorted, validated agent names.
    pub names: Vec<String>,
}

impl AgentCatalog {
    /// Discover agents in `root`. Missing dir → empty catalog (no agents).
    pub fn for_root(root: &Path) -> Self {
        let dir = agents_dir(root);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Self::default();
        };
        let mut names: Vec<String> = entries
            .flatten()
            .filter_map(|entry| {
                let name = entry.file_name().to_str()?.to_string();
                if !valid_name(&name) {
                    return None;
                }
                agent_prompt_path(root, &name).is_file().then_some(name)
            })
            .collect();
        names.sort();
        Self { names }
    }

    /// Resolve a flag token (without the `-`) to a canonical agent name:
    /// the full name always works; a single letter works while exactly one
    /// agent starts with it.
    pub fn resolve_flag(&self, token: &str) -> Option<&str> {
        if let Some(name) = self.names.iter().find(|name| *name == token) {
            return Some(name);
        }
        let mut chars = token.chars();
        let (letter, rest) = (chars.next()?, chars.next());
        if rest.is_some() {
            return None;
        }
        let mut matches = self
            .names
            .iter()
            .filter(|name| name.starts_with(letter));
        match (matches.next(), matches.next()) {
            (Some(name), None) => Some(name),
            _ => None,
        }
    }
}

/// A lowercase directory name usable as a flag token: starts with an ascii
/// letter, then letters, digits, or hyphens. Rules out `-1`-style collisions
/// with numeric group flags by construction.
fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub fn agents_dir(root: &Path) -> PathBuf {
    root.join(".indiana").join("agents")
}

pub fn agent_prompt_path(root: &Path, name: &str) -> PathBuf {
    agents_dir(root).join(name).join("SYSTEM_PROMPT.md")
}

/// The system prompt for one agent in `root`. An unreadable or invalid file
/// falls back to the root's default system prompt with a warning — a broken
/// persona file must never silently change whose voice a batch carries.
pub fn system_prompt_for_agent(root: &Path, name: &str) -> SystemPrompt {
    let path = agent_prompt_path(root, name);
    match SystemPrompt::for_path(&path) {
        Ok(prompt) => prompt,
        Err(e) => {
            let mut fallback = SystemPrompt::for_root(root);
            fallback.warnings.push(format!(
                "{}: {e}; falling back to the default system prompt",
                path.display()
            ));
            fallback
        }
    }
}

/// Scaffold the default agents into `.indiana/agents/`. Existing files are
/// left byte-identical (same policy as `SYSTEM_PROMPT.md`).
pub fn scaffold_agents(root: &Path) -> io::Result<()> {
    for (name, body) in EMBEDDED_AGENTS {
        let path = agent_prompt_path(root, name);
        if path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, body)?;
    }
    Ok(())
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
            "indiana-agents-{n}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_agent(root: &Path, name: &str, body: &str) {
        let path = agent_prompt_path(root, name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "---\nstatus: draft\npurpose: test\napproval: pending\nversion: 1\n---\n\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_scaffold_creates_default_agents() {
        let d = tmp();
        scaffold_agents(&d).unwrap();
        let catalog = AgentCatalog::for_root(&d);
        assert_eq!(catalog.names, vec!["lisa", "mike"]);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_scaffold_does_not_overwrite() {
        let d = tmp();
        write_agent(&d, "mike", "custom mike prompt");
        scaffold_agents(&d).unwrap();
        let text = fs::read_to_string(agent_prompt_path(&d, "mike")).unwrap();
        assert!(text.contains("custom mike prompt"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_resolve_flag_full_name_and_unique_letter() {
        let d = tmp();
        scaffold_agents(&d).unwrap();
        let catalog = AgentCatalog::for_root(&d);
        assert_eq!(catalog.resolve_flag("mike"), Some("mike"));
        assert_eq!(catalog.resolve_flag("m"), Some("mike"));
        assert_eq!(catalog.resolve_flag("l"), Some("lisa"));
        assert_eq!(catalog.resolve_flag("x"), None);
        assert_eq!(catalog.resolve_flag("mi"), None, "prefixes never resolve");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_ambiguous_letter_stops_resolving() {
        let d = tmp();
        write_agent(&d, "mike", "one");
        write_agent(&d, "marketing-mary", "two");
        let catalog = AgentCatalog::for_root(&d);
        assert_eq!(catalog.resolve_flag("m"), None, "two agents start with m");
        assert_eq!(catalog.resolve_flag("mike"), Some("mike"));
        assert_eq!(catalog.resolve_flag("marketing-mary"), Some("marketing-mary"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_invalid_dir_names_are_ignored() {
        let d = tmp();
        write_agent(&d, "mike", "ok");
        // Uppercase and leading-digit names are not flag-safe.
        let bad = agents_dir(&d).join("1agent");
        fs::create_dir_all(&bad).unwrap();
        fs::write(bad.join("SYSTEM_PROMPT.md"), "x").unwrap();
        let catalog = AgentCatalog::for_root(&d);
        assert_eq!(catalog.names, vec!["mike"]);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_agent_system_prompt_resolves_and_falls_back() {
        let d = tmp();
        write_agent(&d, "mike", "INDIANA LOOP — mike's custom voice.");
        let sp = system_prompt_for_agent(&d, "mike");
        assert!(sp.body.contains("mike's custom voice"));
        assert!(sp.warnings.is_empty());

        // Missing agent → default prompt + warning.
        let sp = system_prompt_for_agent(&d, "ghost");
        assert!(!sp.warnings.is_empty());
        assert!(sp.body.starts_with("INDIANA LOOP v"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_embedded_agent_templates_parse() {
        let d = tmp();
        scaffold_agents(&d).unwrap();
        for (name, _) in EMBEDDED_AGENTS {
            let sp = system_prompt_for_agent(&d, name);
            assert!(sp.warnings.is_empty(), "{name}: {:?}", sp.warnings);
            assert!(sp.body.starts_with("INDIANA LOOP v"), "{name}");
        }
        fs::remove_dir_all(d).ok();
    }
}
