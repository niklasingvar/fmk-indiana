//! Shared compiled payload. Copy renders this; MCP returns it as structure.

use crate::cursor;
use crate::index::{Index, Located};
use crate::markers::{kind_matches_filter, long_name, Kind};
use crate::parser::Status;
use crate::scope::ScopeKind;
use crate::templates::TemplateCatalog;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledPayload {
    pub markers: Vec<CompiledMarker>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
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

/// Options that filter what `compile_with_options` includes.
/// The core computes; faces supply options (IN_PRINCIPLES.md: core computes, faces render).
#[derive(Debug, Clone, Default)]
pub struct CompileOptions {
    /// Only include markers matching this kind. None → all kinds.
    /// `action`/`todo` are aliases (see `markers::kind_matches_filter`).
    pub kind: Option<Kind>,
    /// Exclude markers whose `cursor::identity` is in this set. None → no exclusion.
    /// Used by `--latest` to skip already-copied markers.
    pub copied: Option<HashSet<String>>,
    /// Root folders for template lookup. None → embedded templates only.
    pub roots: Option<Vec<PathBuf>>,
}

/// Compile markers from the index, applying optional filters and per-root
/// template catalogs when `options.roots` is set.
pub fn compile_with_options(index: &Index, options: &CompileOptions) -> CompiledPayload {
    let embedded = TemplateCatalog::embedded();
    let root_catalogs: Option<HashMap<PathBuf, TemplateCatalog>> =
        options.roots.as_ref().map(|roots| {
            roots
                .iter()
                .map(|root| (root.clone(), TemplateCatalog::for_root(root)))
                .collect()
        });
    let warnings = root_catalogs
        .as_ref()
        .map(|cats| cats.values().flat_map(|c| c.warnings.clone()).collect())
        .unwrap_or_else(|| embedded.warnings.clone());
    let markers: Vec<CompiledMarker> = index
        .markers
        .iter()
        .filter(|m| match options.kind {
            Some(filter) => kind_matches_filter(filter, m.kind),
            None => true,
        })
        .filter(|m| match &options.copied {
            Some(set) => !set.contains(&cursor::identity(m)),
            None => true,
        })
        .map(|marker| {
            let catalog = match &root_catalogs {
                Some(cats) => owning_root(&marker.path, options.roots.as_ref().unwrap())
                    .and_then(|root| cats.get(root))
                    .unwrap_or(&embedded),
                None => &embedded,
            };
            compile_marker(marker, &catalog.prompts)
        })
        .collect();
    CompiledPayload { markers, warnings }
}

/// Loop preamble prepended to every non-empty rendered payload. Directs the
/// agent into `.indiana/context-model/` (read protocol) and instructs the
/// write-back (log entry + montmartre focus.md). Embedded only — not
/// overridable per root.
const PAYLOAD_PREAMBLE: &str = include_str!("../templates/preamble.md");

pub fn render_text(payload: &CompiledPayload) -> String {
    let markers = payload
        .markers
        .iter()
        .map(|marker| {
            format!(
                "{}:{} [{}]\n{}\n\n{}\n",
                marker.path.display(),
                marker.line,
                long_name(marker.kind),
                marker.compiled_prompt,
                marker.scope_content
            )
        })
        .collect::<Vec<_>>()
        .join("\n---\n");
    if markers.is_empty() {
        return markers;
    }
    format!("{PAYLOAD_PREAMBLE}\n---\n{markers}")
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
        Kind::Question => {
            if let Some(message) = &marker.message {
                apply_message(template(templates, "question"), Some(message))
            } else {
                template(templates, "question_empty")
            }
        }
        _ => apply_message(
            template(templates, long_name(marker.kind)),
            marker.message.as_deref(),
        ),
    }
}

fn template(templates: &HashMap<String, String>, key: &str) -> String {
    templates
        .get(key)
        .unwrap_or_else(|| panic!("missing prompt template: {key}"))
        .to_string()
}

fn apply_message(template: String, message: Option<&str>) -> String {
    template.replace("{message}", message.unwrap_or_default())
}

fn owning_root<'a>(path: &Path, roots: &'a [PathBuf]) -> Option<&'a PathBuf> {
    roots
        .iter()
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
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
        let catalog = TemplateCatalog::embedded();
        assert!(catalog.prompts.contains_key("hate"));
        assert!(catalog.prompts.contains_key("question_empty"));
    }

    #[test]
    fn test_compile_hate() {
        let (d, idx) = index("bad line ::h\n");
        let payload = compile_with_options(&idx, &CompileOptions::default());
        assert!(payload.markers[0]
            .compiled_prompt
            .contains("numbered list why he hates it"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_fix() {
        let (d, idx) = index("buggy ::fix the loop condition\n");
        let payload = compile_with_options(&idx, &CompileOptions::default());
        assert_eq!(
            payload.markers[0].compiled_prompt,
            "Fix this. the loop condition"
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_delete() {
        let (d, idx) = index("junk ::d the dead section\n");
        let payload = compile_with_options(&idx, &CompileOptions::default());
        assert_eq!(payload.markers[0].kind, Kind::Delete);
        assert_eq!(
            payload.markers[0].compiled_prompt,
            "Take action on this and delete the targeted content. Confirm with the user before deleting: the dead section"
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_prompt() {
        let (d, idx) = index("refactor this ::prompt extract the helper\n");
        let payload = compile_with_options(&idx, &CompileOptions::default());
        assert_eq!(payload.markers[0].kind, Kind::Prompt);
        assert_eq!(
            payload.markers[0].compiled_prompt,
            "Run the code agent directly on this. extract the helper"
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_question() {
        let (d, idx) = index("hard ::question why?\n");
        let payload = compile_with_options(&idx, &CompileOptions::default());
        assert_eq!(
            payload.markers[0].compiled_prompt,
            "The user asks: why?. Answer it."
        );
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_scope_in_bundle() {
        let (d, idx) = index("Fix this ::f rename\n");
        let payload = compile_with_options(&idx, &CompileOptions::default());
        assert_eq!(payload.markers[0].scope_content, "Fix this");
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_render_text_prepends_loop_preamble() {
        let (d, idx) = index("buggy ::fix it\n");
        let rendered = render_text(&compile_with_options(&idx, &CompileOptions::default()));
        assert!(rendered.starts_with("INDIANA LOOP"));
        assert!(rendered.contains(".indiana/context-model/CONTEXT-MODEL.md"));
        assert!(rendered.contains(".indiana/montmartre/focus.md"));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_render_text_empty_payload_has_no_preamble() {
        let payload = CompiledPayload {
            markers: Vec::new(),
            warnings: Vec::new(),
        };
        assert_eq!(render_text(&payload), "");
    }

    #[test]
    fn test_copy_all_commands() {
        let (d, idx) = index("one ::h\ntwo ::fix it\nthree ::question why\n");
        let rendered = render_text(&compile_with_options(&idx, &CompileOptions::default()));
        assert!(rendered.contains("hate"));
        assert!(rendered.contains("Fix this. it"));
        assert!(rendered.contains("The user asks: why. Answer it."));
        fs::remove_dir_all(d).ok();
    }

    // ── compile_with_options kind filter ──

    #[test]
    fn test_compile_kind_note_only() {
        let (d, idx) = index("::h\n::note remember this\n::fix tighten\n");
        let opts = CompileOptions {
            kind: Some(Kind::Note),
            copied: None,
            roots: None,
        };
        let payload = compile_with_options(&idx, &opts);
        assert_eq!(payload.markers.len(), 1);
        assert_eq!(payload.markers[0].kind, Kind::Note);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_kind_action_includes_todo() {
        let (d, idx) = index("::h\n::action fix this\n::todo remember\n::note log\n::love\n");
        let opts = CompileOptions {
            kind: Some(Kind::Action),
            copied: None,
            roots: None,
        };
        let payload = compile_with_options(&idx, &opts);
        assert_eq!(payload.markers.len(), 2);
        let kinds: Vec<Kind> = payload.markers.iter().map(|m| m.kind).collect();
        assert!(kinds.contains(&Kind::Action));
        assert!(kinds.contains(&Kind::Todo));
        assert!(!kinds.contains(&Kind::Hate));
        assert!(!kinds.contains(&Kind::Note));
        assert!(!kinds.contains(&Kind::Love));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_no_kind_filter_equals_compile() {
        let (d, idx) = index("::h\n::fix tighten\n::question why\n");
        let all = compile_with_options(&idx, &CompileOptions::default());
        let filtered = compile_with_options(&idx, &CompileOptions::default());
        assert_eq!(all.markers.len(), filtered.markers.len());
        fs::remove_dir_all(d).ok();
    }

    // ── compile_with_options copied (--latest) filter ──

    #[test]
    fn test_compile_copied_excludes_matching() {
        let (d, idx) = index("::h\n::fix tighten\n::question why\n");
        let hate_id = crate::cursor::identity(&idx.markers[0]);
        let mut copied = HashSet::new();
        copied.insert(hate_id.clone());
        let opts = CompileOptions {
            kind: None,
            copied: Some(copied),
            roots: None,
        };
        let payload = compile_with_options(&idx, &opts);
        assert_eq!(payload.markers.len(), 2, "should exclude the hate marker");
        let kinds: Vec<Kind> = payload.markers.iter().map(|m| m.kind).collect();
        assert!(!kinds.contains(&Kind::Hate));
        assert!(kinds.contains(&Kind::Fix));
        assert!(kinds.contains(&Kind::Question));
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_copied_empty_set_copies_all() {
        let (d, idx) = index("::h\n::fix tighten\n");
        let opts = CompileOptions {
            kind: None,
            copied: Some(HashSet::new()),
            roots: None,
        };
        let payload = compile_with_options(&idx, &opts);
        assert_eq!(payload.markers.len(), 2);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_copied_none_copies_all() {
        let (d, idx) = index("::h\n::fix tighten\n");
        let opts = CompileOptions {
            kind: None,
            copied: None,
            roots: None,
        };
        let payload = compile_with_options(&idx, &opts);
        assert_eq!(payload.markers.len(), 2);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_copied_with_kind_composes() {
        let (d, idx) = index("::h\n::action ship it\n::todo remember\n::note log\n");
        // Mark the action as previously copied, but filter to action kind.
        let action_id =
            crate::cursor::identity(idx.markers.iter().find(|m| m.kind == Kind::Action).unwrap());
        let mut copied = HashSet::new();
        copied.insert(action_id);
        let opts = CompileOptions {
            kind: Some(Kind::Action),
            copied: Some(copied),
            roots: None,
        };
        let payload = compile_with_options(&idx, &opts);
        // Only todo should remain (action was already copied, action is excluded).
        assert_eq!(payload.markers.len(), 1);
        assert_eq!(payload.markers[0].kind, Kind::Todo);
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_copied_none_equals_default() {
        let (d, idx) = index("::h\n::fix tighten\n::question why\n");
        let all = compile_with_options(&idx, &CompileOptions::default());
        let explicit = compile_with_options(
            &idx,
            &CompileOptions {
                kind: None,
                copied: None,
                roots: None,
            },
        );
        assert_eq!(all.markers.len(), explicit.markers.len());
        fs::remove_dir_all(d).ok();
    }

    #[test]
    fn test_compile_with_roots_uses_owning_root_template() {
        let (a, idx_a) = index("buggy ::fix tighten\n");
        let (b, idx_b) = index("buggy ::fix tighten\n");
        let prompt = a.join(".indiana/indianas/fix/prompt.md");
        fs::create_dir_all(prompt.parent().unwrap()).unwrap();
        fs::write(
            prompt,
            "---\nstatus: draft\npurpose: test\napproval: pending\ncommand: fix\ncommand_type: test\n---\n\nRepair this. {message}\n",
        )
        .unwrap();
        let mut idx = Index::default();
        idx.markers.extend(idx_a.markers);
        idx.markers.extend(idx_b.markers);
        let payload = compile_with_options(&idx, &CompileOptions { roots: Some(vec![a.clone(), b.clone()]), ..Default::default() });
        let prompts: Vec<String> = payload
            .markers
            .iter()
            .map(|marker| marker.compiled_prompt.clone())
            .collect();
        assert!(prompts.contains(&"Repair this. tighten".to_string()));
        assert!(prompts.contains(&"Fix this. tighten".to_string()));
        fs::remove_dir_all(a).ok();
        fs::remove_dir_all(b).ok();
    }

    #[test]
    fn test_compile_with_roots_bad_template_warns_and_falls_back() {
        let (d, idx) = index("buggy ::fix tighten\n");
        let prompt = d.join(".indiana/indianas/fix/prompt.md");
        fs::create_dir_all(prompt.parent().unwrap()).unwrap();
        fs::write(
            prompt,
            "---\nstatus: draft\npurpose: test\napproval: pending\ncommand: note\ncommand_type: test\n---\n\nRepair this. {message}\n",
        )
        .unwrap();
        let payload = compile_with_options(&idx, &CompileOptions { roots: Some(vec![d.clone()]), ..Default::default() });
        assert_eq!(payload.markers[0].compiled_prompt, "Fix this. tighten");
        assert_eq!(payload.warnings.len(), 1);
        fs::remove_dir_all(d).ok();
    }
}
