//! Folder-local command templates. Embedded prompts are the default; a
//! monitored root may override them with `.indiana/<command>/prompt.md`.

use crate::markers::{Kind, Msg, TABLE};
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
    let prompts = embedded_prompts();
    for spec in TABLE {
        let dir = root.join(".indiana").join(spec.long);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("prompt.md");
        if path.exists() {
            continue;
        }
        let body = prompts
            .get(spec.long)
            .unwrap_or_else(|| panic!("missing prompt template: {}", spec.long));
        std::fs::write(path, scaffold(spec.long, spec.kind, spec.msg, body))?;
    }
    Ok(())
}

fn embedded_prompts() -> HashMap<String, String> {
    toml::from_str::<PromptFile>(include_str!("../prompts.toml"))
        .expect("embedded prompts.toml must parse")
        .prompts
}

fn template_path(root: &Path, command: &str) -> PathBuf {
    root.join(".indiana").join(command).join("prompt.md")
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

fn scaffold(command: &str, kind: Kind, msg: Msg, body: &str) -> String {
    format!(
        "---\nstatus: draft\npurpose: Folder-local prompt template and behavior for ::{command}.\napproval: pending\ncommand: {command}\ncommand_type: {}\nmessage: {}\n---\n\n{body}\n",
        command_type(kind),
        message_contract(msg),
    )
}

fn command_type(kind: Kind) -> &'static str {
    match kind {
        Kind::Fix | Kind::Elaborate => "agent_directive",
        Kind::Question => "agent_explains",
        Kind::Hate | Kind::Love | Kind::Keep => "reaction",
        Kind::Note => "user_context",
        Kind::Action | Kind::Todo => "user_task",
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
        assert!(d.join(".indiana/fix/prompt.md").exists());
        assert!(d.join(".indiana/question/prompt.md").exists());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_init_folder_indiana_does_not_overwrite() {
        let d = tmp();
        let path = d.join(".indiana/fix/prompt.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "custom").unwrap();
        init_folder_indiana(&d).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "custom");
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
}
