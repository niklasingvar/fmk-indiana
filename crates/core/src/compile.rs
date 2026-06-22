//! Shared compiled payload. Copy renders this; MCP returns it as structure.

use crate::index::{Index, Located};
use crate::markers::Kind;
use crate::parser::Status;
use crate::scope::ScopeKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPayload {
    pub markers: Vec<CompiledMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledMarker {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub kind: Kind,
    pub raw_token: String,
    pub compiled_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    pub path: PathBuf,
    pub line: usize,
    pub scope_kind: ScopeKind,
    pub scope_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

#[derive(Debug, Deserialize)]
struct PromptFile {
    prompts: HashMap<String, String>,
}

pub fn compile(index: &Index) -> CompiledPayload {
    let templates = templates();
    CompiledPayload {
        markers: index
            .markers
            .iter()
            .map(|marker| compile_marker(marker, &templates))
            .collect(),
    }
}

pub fn render_text(payload: &CompiledPayload) -> String {
    payload
        .markers
        .iter()
        .map(|marker| {
            format!(
                "{}:{} [{}]\n{}\n\n{}\n",
                marker.path.display(),
                marker.line,
                kind_key(marker.kind),
                marker.compiled_prompt,
                marker.scope_content
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n")
}

fn compile_marker(marker: &Located, templates: &HashMap<String, String>) -> CompiledMarker {
    CompiledMarker {
        id: marker.id.clone(),
        kind: marker.kind,
        raw_token: marker.raw_token.clone(),
        compiled_prompt: prompt(marker, templates),
        message: marker.message.clone(),
        path: marker.path.clone(),
        line: marker.line,
        scope_kind: marker.scope.kind.clone(),
        scope_content: marker.scope.content.clone(),
        status: marker.status,
    }
}

fn prompt(marker: &Located, templates: &HashMap<String, String>) -> String {
    match marker.kind {
        Kind::Fix | Kind::Elaborate => {
            let mut out = template(templates, kind_key(marker.kind));
            if let Some(message) = &marker.message {
                out.push(' ');
                out.push_str(message);
            }
            out
        }
        Kind::Question => {
            if let Some(message) = &marker.message {
                template(templates, "question").replace("{message}", message)
            } else {
                template(templates, "question_empty")
            }
        }
        _ => template(templates, kind_key(marker.kind))
            .replace("{message}", marker.message.as_deref().unwrap_or_default()),
    }
}

fn templates() -> HashMap<String, String> {
    toml::from_str::<PromptFile>(include_str!("../prompts.toml"))
        .expect("embedded prompts.toml must parse")
        .prompts
}

fn template(templates: &HashMap<String, String>, key: &str) -> String {
    templates
        .get(key)
        .unwrap_or_else(|| panic!("missing prompt template: {key}"))
        .to_string()
}

fn kind_key(kind: Kind) -> &'static str {
    match kind {
        Kind::Question => "question",
        Kind::Hate => "hate",
        Kind::Love => "love",
        Kind::Keep => "keep",
        Kind::Fix => "fix",
        Kind::Elaborate => "elaborate",
        Kind::Note => "note",
        Kind::Action => "action",
        Kind::Todo => "todo",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Index;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "indiana-compile-{n}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn index(body: &str) -> (PathBuf, Index) {
        let d = tmp();
        fs::write(d.join("doc.md"), body).unwrap();
        let idx = Index::build(&d);
        (d, idx)
    }

    #[test]
    fn test_prompt_templates_external() {
        let parsed: PromptFile = toml::from_str(include_str!("../prompts.toml")).unwrap();
        assert!(parsed.prompts.contains_key("hate"));
        assert!(parsed.prompts.contains_key("question_empty"));
    }

    #[test]
    fn test_compile_hate() {
        let (d, idx) = index("bad line ::h\n");
        let payload = compile(&idx);
        assert!(payload.markers[0]
            .compiled_prompt
            .contains("numbered list why he hates it"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_fix() {
        let (d, idx) = index("buggy ::fix the loop condition\n");
        let payload = compile(&idx);
        assert_eq!(
            payload.markers[0].compiled_prompt,
            "Fix this. the loop condition"
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_question() {
        let (d, idx) = index("hard ::question why?\n");
        let payload = compile(&idx);
        assert_eq!(
            payload.markers[0].compiled_prompt,
            "The user asks: why?. Answer it."
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_scope_in_bundle() {
        let (d, idx) = index("Fix this ::f rename\n");
        let payload = compile(&idx);
        assert_eq!(payload.markers[0].scope_content, "Fix this");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_copy_all_commands() {
        let (d, idx) = index("one ::h\ntwo ::fix it\nthree ::question why\n");
        let rendered = render_text(&compile(&idx));
        assert!(rendered.contains("hate"));
        assert!(rendered.contains("Fix this. it"));
        assert!(rendered.contains("The user asks: why. Answer it."));
        fs::remove_dir_all(d).ok();
    }
}
