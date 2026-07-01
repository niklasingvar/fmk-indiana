//! Folder-local command templates. Embedded prompts are the default; a
//! monitored root may override them with `.indiana/indianas/<command>/prompt.md`.
//!
//! `init_folder_indiana` also scaffolds the sibling meta folders every
//! monitored root carries: `.indiana/context-model/` (state, direction, rules)
//! and `.indiana/montmartre/` (project management: actions, notes, focus).
//! Their seeded content lives in `crates/core/scaffold/` and is the source of
//! truth — edit those files to change what new monitored roots start with.

use crate::markers::{Msg, TABLE};
use serde::Deserialize;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateCatalog {
    pub prompts: HashMap<String, String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct PromptFile {
    prompts: HashMap<String, String>,
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
    scaffold_meta(root)?;
    Ok(())
}

/// Overwrite every `.indiana/indianas/<command>/prompt.md` with the embedded
/// default. Existing user edits to command templates are discarded. Meta
/// folders (`context-model/`, `montmartre/`) are not touched — replace is
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

/// Write the 9 `indianas/<command>/prompt.md` files from embedded defaults.
/// `overwrite` false skips existing files (refresh/init); true rewrites them
/// (replace).
fn write_command_templates(root: &Path, overwrite: bool) -> io::Result<()> {
    let prompts = embedded_prompts();
    for spec in TABLE {
        let dir = root.join(".indiana").join("indianas").join(spec.long);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("prompt.md");
        if !overwrite && path.exists() {
            continue;
        }
        let body = prompts
            .get(spec.long)
            .unwrap_or_else(|| panic!("missing prompt template: {}", spec.long));
        std::fs::write(path, scaffold(spec.long, spec.command_type, spec.msg, body))?;
    }
    Ok(())
}

/// Scaffold the sibling meta folders: `context-model/` and `montmartre/`.
/// Content is `include_str!`-ed from `crates/core/scaffold/`, which is the
/// source of truth. Existing files are left byte-identical.
fn scaffold_meta(root: &Path) -> io::Result<()> {
    let context_model = root.join(".indiana").join("context-model");
    std::fs::create_dir_all(&context_model)?;
    let gitkeep = context_model.join(".gitkeep");
    if !gitkeep.exists() {
        std::fs::write(gitkeep, include_str!("../scaffold/context-model/.gitkeep"))?;
    }

    let montmartre = root.join(".indiana").join("montmartre");
    std::fs::create_dir_all(&montmartre)?;
    for (name, body) in [
        ("README.md", include_str!("../scaffold/montmartre/README.md")),
        ("actions.md", include_str!("../scaffold/montmartre/actions.md")),
        ("notes.md", include_str!("../scaffold/montmartre/notes.md")),
        ("focus.md", include_str!("../scaffold/montmartre/focus.md")),
    ] {
        let path = montmartre.join(name);
        if !path.exists() {
            std::fs::write(path, body)?;
        }
    }
    Ok(())
}

fn embedded_prompts() -> HashMap<String, String> {
    toml::from_str::<PromptFile>(include_str!("../prompts.toml"))
        .expect("embedded prompts.toml must parse")
        .prompts
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

fn scaffold(command: &str, command_type: &str, msg: Msg, body: &str) -> String {
    format!(
        "---\nstatus: draft\npurpose: Folder-local prompt template and behavior for ::{command}.\napproval: pending\ncommand: {command}\ncommand_type: {command_type}\nmessage: {}\n---\n\n{}\n\n{body}\n",
        message_contract(msg),
        heading(command),
    )
}

/// One-line `#` heading prepended to each scaffolded `prompt.md` for human
/// readability. `first_paragraph` skips `#` lines, so this never reaches the
/// compiled prompt — it only documents the file for the person editing it.
fn heading(command: &str) -> &'static str {
    match command {
        "fix" => "# ::fix — agent directive: fix this. Message refines how.",
        "elaborate" => "# ::elaborate — agent directive: act on this and elaborate the change.",
        "question" => "# ::question — agent explains: answer the user's question.",
        "hate" => "# ::hate — reaction: user dislikes this (no message).",
        "love" => "# ::love — reaction: user likes this; preserve the pattern.",
        "keep" => "# ::keep — reaction: freeze; do not change this.",
        "note" => "# ::note — user context (passthrough): the message is the prompt.",
        "action" => "# ::action — user task (passthrough): the message is the prompt.",
        "todo" => "# ::todo — user task (passthrough): the message is the prompt.",
        "delete" => "# ::delete — agent gated directive: delete targeted content, then check in with the user before acting.",
        _ => "# (unknown command)",
    }
}

fn message_contract(msg: Msg) -> &'static str {
    match msg {
        Msg::None => "none",
        Msg::Optional => "optional",
        Msg::Required => "required",
    }
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
        assert!(d.join(".indiana/context-model/.gitkeep").exists());
        assert!(d.join(".indiana/montmartre/README.md").exists());
        assert!(d.join(".indiana/montmartre/actions.md").exists());
        assert!(d.join(".indiana/montmartre/notes.md").exists());
        assert!(d.join(".indiana/montmartre/focus.md").exists());
        assert_eq!(
            fs::read_to_string(d.join(".indiana/montmartre/focus.md")).unwrap(),
            "# Focus\n"
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
        assert!(file.contains("# ::action —"), "heading present");
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
        let focus = d.join(".indiana/montmartre/focus.md");
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
        let focus = d.join(".indiana/montmartre/focus.md");
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

    // IN_PRINCIPLES.md: this repo authors the defaults. Every TABLE row has a
    // `.indiana/indianas/<long>/prompt.md` in this repository whose
    // frontmatter and body match the marker metadata and `prompts.toml`.
    // Skipped downstream where the repo `.indiana` is absent (a packaged crate
    // build carries only the crate dir, not the workspace `.indiana`).
    #[test]
    fn test_repo_indianas_match_embedded_defaults() {
        let ws = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let indianas = ws.join(".indiana").join("indianas");
        if !indianas.is_dir() {
            return;
        }
        let prompts = embedded_prompts();
        for spec in TABLE {
            let path = template_path(&ws, spec.long);
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("missing repo template: {}", path.display()));
            let (frontmatter, body) = split_frontmatter(&text)
                .unwrap_or_else(|_| panic!("bad frontmatter in {}", path.display()));
            let parsed: Frontmatter = serde_yml::from_str(frontmatter)
                .unwrap_or_else(|_| panic!("bad yaml in {}", path.display()));
            assert_eq!(parsed.command, spec.long, "command mismatch in {}", path.display());
            assert_eq!(
                parsed.command_type, spec.command_type,
                "command_type mismatch in {}",
                path.display()
            );
            assert_eq!(
                parsed.message.as_deref(),
                Some(message_contract(spec.msg)),
                "message mismatch in {}",
                path.display()
            );
            let para = first_paragraph(body)
                .unwrap_or_else(|| panic!("missing body in {}", path.display()));
            assert_eq!(
                para.as_str(),
                prompts[spec.long].as_str(),
                "body drift between {} and prompts.toml",
                path.display()
            );
        }
    }
}
