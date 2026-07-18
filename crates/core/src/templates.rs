//! Folder-local command templates. Embedded prompts are the default; a
//! monitored root may override them with `.indiana/indianas/<command>/prompt.md`.
//!
//! `init_folder_indiana` also scaffolds the sibling meta folders every
//! monitored root carries: `.indiana/context-model/` (state, direction, rules)
//! and `.indiana/chief-of-staff/` (project management: actions, notes, focus).
//!
//! `crates/core/templates/` is the single authoring source for everything a
//! monitored root starts with: full `prompt.md` files under `indianas/`, the
//! versioned `system_prompt.md`, and meta folder seeds. Files are embedded at
//! compile time and written verbatim — edit them to change what users receive.
//! This repository's own `.indiana/` is a dogfood instance, not a source; it
//! may diverge freely.

use crate::markers::TABLE;
use crate::system_prompt::{system_prompt_path, SystemPrompt};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

/// One embedded `prompt.md` per marker command, written verbatim into
/// `.indiana/indianas/<command>/prompt.md`. A unit test pins each file's
/// frontmatter to its marker TABLE row.
const EMBEDDED_TEMPLATES: &[(&str, &str)] = &[
    ("question", include_str!("../templates/indianas/question/prompt.md")),
    ("hate", include_str!("../templates/indianas/hate/prompt.md")),
    ("love", include_str!("../templates/indianas/love/prompt.md")),
    ("keep", include_str!("../templates/indianas/keep/prompt.md")),
    ("fix", include_str!("../templates/indianas/fix/prompt.md")),
    ("elaborate", include_str!("../templates/indianas/elaborate/prompt.md")),
    ("note", include_str!("../templates/indianas/note/prompt.md")),
    ("action", include_str!("../templates/indianas/action/prompt.md")),
    ("todo", include_str!("../templates/indianas/todo/prompt.md")),
    ("delete", include_str!("../templates/indianas/delete/prompt.md")),
    ("prompt", include_str!("../templates/indianas/prompt/prompt.md")),
];

/// Variant prompt compiled when a `::question` marker carries no message.
/// Embedded only — not written into instances and not overridable per root
/// (`for_root` reads only `prompt.md` files).
const QUESTION_EMPTY_TEMPLATE: &str =
    include_str!("../templates/indianas/question/prompt_empty.md");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateCatalog {
    pub prompts: HashMap<String, String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    status: String,
    purpose: String,
    approval: String,
    command: String,
    command_type: String,
    #[allow(dead_code)]
    message: Option<String>,
}

impl TemplateCatalog {
    pub fn embedded() -> Self {
        Self {
            prompts: embedded_prompts(),
            warnings: Vec::new(),
        }
    }

    pub fn for_root(root: &Path) -> Self {
        let mut catalog = Self::embedded();
        for spec in TABLE {
            let path = template_path(root, spec.long);
            if !path.exists() {
                continue;
            }
            match read_template(&path, spec.long) {
                Ok(template) => {
                    catalog.prompts.insert(spec.long.to_string(), template);
                }
                Err(e) => catalog.warnings.push(format!("{}: {e}", path.display())),
            }
        }
        catalog
    }
}

pub fn init_folder_indiana(root: &Path) -> io::Result<()> {
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", root.display()),
        ));
    }
    write_command_templates(root, false)?;
    scaffold_system_prompt(root)?;
    scaffold_meta(root)?;
    Ok(())
}

/// Scaffold `.indiana/SYSTEM_PROMPT.md` from the embedded authoring source.
/// Existing files are left byte-identical (refresh adds missing only).
fn scaffold_system_prompt(root: &Path) -> io::Result<()> {
    let path = system_prompt_path(root);
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, SystemPrompt::embedded_raw())
}

/// Overwrite every `.indiana/indianas/<command>/prompt.md` with the embedded
/// default. Existing user edits to command templates are discarded. Meta
/// folders (`context-model/`, `chief-of-staff/`) are not touched — replace is
/// scoped to command templates only. Backed by `indiana templates replace`.
pub fn replace_folder_indiana(root: &Path) -> io::Result<()> {
    if !root.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", root.display()),
        ));
    }
    write_command_templates(root, true)?;
    Ok(())
}

/// Write every `indianas/<command>/prompt.md` verbatim from the embedded
/// template files. `overwrite` false skips existing files (refresh/init);
/// true rewrites them (replace).
fn write_command_templates(root: &Path, overwrite: bool) -> io::Result<()> {
    for spec in TABLE {
        let dir = root.join(".indiana").join("indianas").join(spec.long);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("prompt.md");
        if !overwrite && path.exists() {
            continue;
        }
        std::fs::write(path, embedded_template(spec.long))?;
    }
    Ok(())
}

/// Scaffold the sibling meta folders: `context-model/` and `chief-of-staff/`.
/// Content is `include_str!`-ed from `crates/core/templates/`, which is the
/// source of truth. Existing files are left byte-identical.
fn scaffold_meta(root: &Path) -> io::Result<()> {
    let context_model = root.join(".indiana").join("context-model");
    for (name, body) in [
        (
            "CONTEXT-MODEL.md",
            include_str!("../templates/context-model/CONTEXT-MODEL.md"),
        ),
        ("index.md", include_str!("../templates/context-model/index.md")),
        ("log.md", include_str!("../templates/context-model/log.md")),
        (
            "purpose/PURPOSE.md",
            include_str!("../templates/context-model/purpose/PURPOSE.md"),
        ),
        (
            "learnings/INBOX.md",
            include_str!("../templates/context-model/learnings/INBOX.md"),
        ),
    ] {
        let path = context_model.join(name);
        std::fs::create_dir_all(path.parent().unwrap())?;
        if !path.exists() {
            std::fs::write(path, body)?;
        }
    }

    let chief_of_staff = root.join(".indiana").join("chief-of-staff");
    std::fs::create_dir_all(&chief_of_staff)?;
    for (name, body) in [
        ("README.md", include_str!("../templates/chief-of-staff/README.md")),
        ("tasks.md", crate::cos::TASKS_SEED),
        ("log.md", crate::cos::LOG_SEED),
        ("notes.md", include_str!("../templates/chief-of-staff/notes.md")),
        ("focus.md", include_str!("../templates/chief-of-staff/focus.md")),
    ] {
        let path = chief_of_staff.join(name);
        if !path.exists() {
            std::fs::write(path, body)?;
        }
    }
    Ok(())
}

fn embedded_template(command: &str) -> &'static str {
    EMBEDDED_TEMPLATES
        .iter()
        .find(|(name, _)| *name == command)
        .map(|(_, text)| *text)
        .unwrap_or_else(|| panic!("missing embedded template: {command}"))
}

fn embedded_prompts() -> HashMap<String, String> {
    EMBEDDED_TEMPLATES
        .iter()
        .map(|(name, text)| (*name, *text))
        .chain([("question_empty", QUESTION_EMPTY_TEMPLATE)])
        .map(|(name, text)| {
            let (_, body) = split_frontmatter(text)
                .unwrap_or_else(|e| panic!("embedded template {name}: {e}"));
            let para = first_paragraph(body)
                .unwrap_or_else(|| panic!("embedded template {name}: missing prompt body"));
            (name.to_string(), para)
        })
        .collect()
}

fn template_path(root: &Path, command: &str) -> PathBuf {
    root.join(".indiana")
        .join("indianas")
        .join(command)
        .join("prompt.md")
}

fn read_template(path: &Path, command: &str) -> Result<String, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&text)?;
    let parsed: Frontmatter = serde_yml::from_str(frontmatter).map_err(|e| e.to_string())?;
    validate_frontmatter(&parsed, command)?;
    first_paragraph(body).ok_or_else(|| "missing prompt body".to_string())
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

fn validate_frontmatter(f: &Frontmatter, command: &str) -> Result<(), String> {
    for (name, value) in [
        ("status", &f.status),
        ("purpose", &f.purpose),
        ("approval", &f.approval),
        ("command_type", &f.command_type),
    ] {
        if value.trim().is_empty() {
            return Err(format!("frontmatter field `{name}` is empty"));
        }
    }
    if f.command != command {
        return Err(format!(
            "frontmatter command `{}` does not match folder `{command}`",
            f.command
        ));
    }
    Ok(())
}

fn first_paragraph(body: &str) -> Option<String> {
    let mut lines = Vec::new();
    let mut started = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if !started {
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            started = true;
        } else if trimmed.is_empty() {
            break;
        }
        lines.push(line);
    }
    let paragraph = lines.join("\n").trim().to_string();
    (!paragraph.is_empty()).then_some(paragraph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markers::Msg;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn message_contract(msg: Msg) -> &'static str {
        match msg {
            Msg::None => "none",
            Msg::Optional => "optional",
            Msg::Required => "required",
        }
    }

    fn tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "indiana-templates-{n}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_template(root: &Path, command: &str, frontmatter_command: &str, body: &str) {
        let path = template_path(root, command);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            format!(
                "---\nstatus: draft\npurpose: test\napproval: pending\ncommand: {frontmatter_command}\ncommand_type: test\n---\n\n{body}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn test_init_folder_indiana_scaffolds_commands() {
        let d = tmp();
        init_folder_indiana(&d).unwrap();
        assert!(d.join(".indiana/indianas/fix/prompt.md").exists());
        assert!(d.join(".indiana/indianas/question/prompt.md").exists());
        assert!(d.join(".indiana/indianas/delete/prompt.md").exists());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_init_folder_indiana_scaffolds_meta_folders() {
        let d = tmp();
        init_folder_indiana(&d).unwrap();
        assert!(d.join(".indiana/context-model/CONTEXT-MODEL.md").exists());
        assert!(d.join(".indiana/context-model/index.md").exists());
        assert!(d.join(".indiana/context-model/log.md").exists());
        assert!(d.join(".indiana/context-model/purpose/PURPOSE.md").exists());
        assert!(d.join(".indiana/context-model/learnings/INBOX.md").exists());
        assert!(d.join(".indiana/chief-of-staff/README.md").exists());
        assert!(d.join(".indiana/chief-of-staff/tasks.md").exists());
        assert!(d.join(".indiana/chief-of-staff/log.md").exists());
        assert!(d.join(".indiana/chief-of-staff/notes.md").exists());
        assert!(d.join(".indiana/chief-of-staff/focus.md").exists());
        assert_eq!(
            fs::read_to_string(d.join(".indiana/chief-of-staff/focus.md")).unwrap(),
            "# Focus\n"
        );
        let system_prompt = d.join(".indiana/SYSTEM_PROMPT.md");
        assert!(system_prompt.exists());
        assert_eq!(
            fs::read_to_string(&system_prompt).unwrap(),
            SystemPrompt::embedded_raw()
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_init_folder_indiana_does_not_overwrite_system_prompt() {
        let d = tmp();
        let path = d.join(".indiana/SYSTEM_PROMPT.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "custom system prompt\n").unwrap();
        init_folder_indiana(&d).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "custom system prompt\n");
        replace_folder_indiana(&d).unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "custom system prompt\n",
            "replace stays scoped to indianas/"
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_scaffold_heading_is_skipped_by_compiler() {
        let d = tmp();
        init_folder_indiana(&d).unwrap();
        // The scaffolded action file opens with a `#` heading, then `{message}`.
        let file = fs::read_to_string(d.join(".indiana/indianas/action/prompt.md")).unwrap();
        assert!(file.starts_with("---\n"), "frontmatter first");
        assert!(file.contains("# ::action"), "heading present");
        assert!(file.contains("\n{message}\n"), "body still {{message}}");
        // The compiler reads only the first non-heading paragraph, so the
        // heading never reaches the compiled prompt.
        let catalog = TemplateCatalog::for_root(&d);
        assert_eq!(catalog.prompts["action"], "{message}");
        assert!(catalog.warnings.is_empty());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_init_folder_indiana_does_not_overwrite() {
        let d = tmp();
        let path = d.join(".indiana/indianas/fix/prompt.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "custom").unwrap();
        init_folder_indiana(&d).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "custom");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_init_folder_indiana_does_not_overwrite_meta() {
        let d = tmp();
        let focus = d.join(".indiana/chief-of-staff/focus.md");
        fs::create_dir_all(focus.parent().unwrap()).unwrap();
        fs::write(&focus, "# my focus\n").unwrap();
        init_folder_indiana(&d).unwrap();
        assert_eq!(fs::read_to_string(&focus).unwrap(), "# my focus\n");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_replace_folder_indiana_overwrites_commands() {
        let d = tmp();
        init_folder_indiana(&d).unwrap();
        let fix = d.join(".indiana/indianas/fix/prompt.md");
        fs::write(&fix, "my custom fix wording").unwrap();
        replace_folder_indiana(&d).unwrap();
        let body = fs::read_to_string(&fix).unwrap();
        assert!(body.contains("Fix this."));
        assert!(!body.contains("my custom fix wording"));
        // Meta folders are left untouched by replace.
        let focus = d.join(".indiana/chief-of-staff/focus.md");
        fs::create_dir_all(focus.parent().unwrap()).unwrap();
        fs::write(&focus, "# my focus\n").unwrap();
        replace_folder_indiana(&d).unwrap();
        assert_eq!(fs::read_to_string(&focus).unwrap(), "# my focus\n");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_root_template_overrides_embedded() {
        let d = tmp();
        write_template(&d, "fix", "fix", "Repair this. {message}");
        let catalog = TemplateCatalog::for_root(&d);
        assert_eq!(catalog.prompts["fix"], "Repair this. {message}");
        assert!(catalog.warnings.is_empty());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_bad_frontmatter_falls_back_with_warning() {
        let d = tmp();
        write_template(&d, "fix", "note", "Repair this. {message}");
        let catalog = TemplateCatalog::for_root(&d);
        assert_eq!(catalog.prompts["fix"], "Fix this. {message}");
        assert_eq!(catalog.warnings.len(), 1);
        fs::remove_dir_all(d).ok();
    }

    // IN_PRINCIPLES.md: `crates/core/templates/indianas/` authors the
    // defaults. Every TABLE row has exactly one embedded template whose
    // frontmatter matches the marker metadata and whose body compiles to a
    // prompt. The repo's own `.indiana/` is a dogfood instance and is
    // deliberately not checked — it may diverge.
    #[test]
    fn test_embedded_templates_match_marker_table() {
        assert_eq!(
            EMBEDDED_TEMPLATES.len(),
            TABLE.len(),
            "one embedded template per marker TABLE row"
        );
        for spec in TABLE {
            let text = embedded_template(spec.long);
            let (frontmatter, body) = split_frontmatter(text)
                .unwrap_or_else(|e| panic!("bad frontmatter in template {}: {e}", spec.long));
            let parsed: Frontmatter = serde_yml::from_str(frontmatter)
                .unwrap_or_else(|e| panic!("bad yaml in template {}: {e}", spec.long));
            assert_eq!(parsed.command, spec.long, "command mismatch in template {}", spec.long);
            assert_eq!(
                parsed.command_type, spec.command_type,
                "command_type mismatch in template {}",
                spec.long
            );
            assert_eq!(
                parsed.message.as_deref(),
                Some(message_contract(spec.msg)),
                "message mismatch in template {}",
                spec.long
            );
            first_paragraph(body)
                .unwrap_or_else(|| panic!("missing body in template {}", spec.long));
        }
    }
}
