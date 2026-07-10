//! Cursor identity — position-independent marker fingerprints for `--latest`.
//! Tracked markers (action / todo) use their injected ID; ephemeral markers
//! use a content hash so they survive line moves but react to edits.
//! The ephemeral fingerprint is computed in memory only, never written to source
//! (IN_IDENTITY.md).

use crate::compile::CompiledMarker;
use crate::index::Located;
use crate::markers::long_name;

/// Compute a position-independent identity for a marker.
/// - Tracked → `id:<the-id>` (stable across edits, moves, renames).
/// - Ephemeral → `fp:<hex-hash>` of path, kind, raw_token, message, scope content.
///   Excludes line number. Move within a file → same identity. Edit text → new.
pub fn identity(located: &Located) -> String {
    if let Some(id) = &located.id {
        return format!("id:{id}");
    }
    fingerprint(
        &located.path.to_string_lossy(),
        long_name(located.kind),
        &located.raw_token,
        located.message.as_deref().unwrap_or(""),
        &located
            .group
            .map(|group| group.to_string())
            .unwrap_or_default(),
        &located.scope.content,
    )
}

/// Identity of a compiled marker — same rule as `identity`, over the post-compile
/// shape. Lets a face record what it delivered without re-finding the `Located`.
pub fn identity_compiled(marker: &CompiledMarker) -> String {
    if let Some(id) = &marker.id {
        return format!("id:{id}");
    }
    fingerprint(
        &marker.path.to_string_lossy(),
        long_name(marker.kind),
        &marker.raw_token,
        marker.message.as_deref().unwrap_or(""),
        &marker
            .group
            .map(|group| group.to_string())
            .unwrap_or_default(),
        &marker.scope_content,
    )
}

// ── helpers ──

fn fingerprint(
    path: &str,
    kind: &str,
    raw_token: &str,
    message: &str,
    group: &str,
    scope: &str,
) -> String {
    let fp = fnv1a_hex(&[
        path.as_bytes(),
        kind.as_bytes(),
        raw_token.as_bytes(),
        message.as_bytes(),
        group.as_bytes(),
        scope.as_bytes(),
    ]);
    format!("fp:{fp}")
}

/// FNV-1a 64-bit hash — deterministic, compact, no external deps.
fn fnv1a_hex(chunks: &[&[u8]]) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for chunk in chunks {
        for &byte in *chunk {
            h ^= byte as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        // Separator between chunks so `("a","bc")` ≠ `("ab","c")`.
        h ^= 0;
        h = h.wrapping_mul(0x100000001b3);
    }
    // 16 hex digits — compact enough for a fingerprint.
    format!("{h:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::Located;
    use crate::markers::Kind;
    use crate::scope::{Scope, ScopeKind};
    use std::path::PathBuf;

    fn located(kind: Kind, raw_token: &str, msg: Option<&str>, content: &str) -> Located {
        Located {
            path: PathBuf::from("/tmp/doc.md"),
            line: 1,
            column: 0,
            kind,
            raw_token: raw_token.to_string(),
            message: msg.map(|s| s.to_string()),
            group: None,
            id: None,
            status: None,
            auto: false,
            scope: Scope {
                kind: ScopeKind::Inline,
                content: content.to_string(),
            },
        }
    }

    #[test]
    fn test_identity_tracked_uses_id() {
        let mut l = located(Kind::Action, "a", Some("ship it"), "do it");
        l.id = Some("bear-mouse".into());
        assert_eq!(identity(&l), "id:bear-mouse");
    }

    #[test]
    fn test_identity_ephemeral_stable_on_move() {
        let a = {
            let mut l = located(Kind::Hate, "h", None, "bad code");
            l.line = 5;
            identity(&l)
        };
        let b = {
            let mut l = located(Kind::Hate, "h", None, "bad code");
            l.line = 99;
            identity(&l)
        };
        assert_eq!(a, b, "fingerprint must ignore line number");
    }

    #[test]
    fn test_identity_ephemeral_changes_on_edit() {
        let a = {
            let mut l = located(Kind::Fix, "f", Some("tighten"), "loop");
            l.line = 10;
            identity(&l)
        };
        let b = {
            let mut l = located(Kind::Fix, "f", Some("loosen"), "loop");
            l.line = 10;
            identity(&l)
        };
        assert_ne!(a, b, "different message → different fingerprint");
    }

    #[test]
    fn test_identity_ephemeral_path_matters() {
        let a = located(Kind::Note, "n", Some("log"), "important");
        let mut b = a.clone();
        b.path = PathBuf::from("/other/doc.md");
        assert_ne!(identity(&a), identity(&b));
    }

    #[test]
    fn test_identity_ephemeral_group_matters() {
        let mut a = located(Kind::Fix, "fix", Some("tighten"), "loop");
        a.group = Some(1);
        let mut b = a.clone();
        b.group = Some(2);
        assert_ne!(identity(&a), identity(&b));
    }

    // identity and identity_compiled must agree for the same marker, so the cursor
    // a copy records (compiled) matches what the filter excludes (located).
    #[test]
    fn test_identity_compiled_matches_located() {
        let l = located(Kind::Hate, "h", None, "bad code");
        let cm = CompiledMarker {
            id: None,
            kind: Kind::Hate,
            raw_token: "h".into(),
            compiled_prompt: "ignored by identity".into(),
            message: None,
            group: None,
            path: l.path.clone(),
            line: 42, // line differs — must not affect identity
            scope_kind: ScopeKind::Inline,
            scope_content: "bad code".into(),
            status: None,
            auto: false,
        };
        assert_eq!(identity(&l), identity_compiled(&cm));
    }
}
