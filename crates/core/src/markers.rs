//! The one marker table (IN_PRINCIPLES.md: one table drives everything).
//! Parser, compiler, and identity all read this; none re-encode the set.
//! Adding a marker is one row here (IN_COMMANDS.md "The set").

/// What a marker means. One variant per row of the IN_COMMANDS.md table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Question,
    Hate,
    Love,
    Keep,
    Fix,
    Elaborate,
    Note,
    Action,
    Todo,
}

/// Whether a kind carries a message (IN_COMMANDS.md: pure reactions take none).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    None,
    Optional,
    Required,
}

/// One row of the table.
pub struct MarkerSpec {
    /// Short tokens, without the `::` prefix (e.g. `"h"`, `"?"`).
    pub shorts: &'static [&'static str],
    /// Long token, without the `::` prefix (e.g. `"hate"`).
    pub long: &'static str,
    pub kind: Kind,
    pub msg: Msg,
    /// Tracked kinds get an injected id (IN_IDENTITY.md: action / todo only).
    pub tracked: bool,
}

/// The table. Source of truth for the grammar.
pub const TABLE: &[MarkerSpec] = &[
    MarkerSpec {
        shorts: &["q", "?"],
        long: "question",
        kind: Kind::Question,
        msg: Msg::Optional,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["h"],
        long: "hate",
        kind: Kind::Hate,
        msg: Msg::None,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["l"],
        long: "love",
        kind: Kind::Love,
        msg: Msg::None,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["k"],
        long: "keep",
        kind: Kind::Keep,
        msg: Msg::None,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["f"],
        long: "fix",
        kind: Kind::Fix,
        msg: Msg::Optional,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["e"],
        long: "elaborate",
        kind: Kind::Elaborate,
        msg: Msg::Optional,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["n"],
        long: "note",
        kind: Kind::Note,
        msg: Msg::Required,
        tracked: false,
    },
    MarkerSpec {
        shorts: &["a"],
        long: "action",
        kind: Kind::Action,
        msg: Msg::Required,
        tracked: true,
    },
    MarkerSpec {
        shorts: &["td"],
        long: "todo",
        kind: Kind::Todo,
        msg: Msg::Required,
        tracked: true,
    },
];

/// Action and Todo are aliases: one tracked, actionable kind under two tokens
/// (IN_IDENTITY.md). Filtering by either token returns both.
fn is_actionable(k: Kind) -> bool {
    matches!(k, Kind::Action | Kind::Todo)
}

/// Does a `--kind` filter (itself a Kind) accept the given kind?
/// The only grouping is the action/todo alias; every other kind matches itself.
pub fn kind_matches_filter(filter: Kind, k: Kind) -> bool {
    filter == k || (is_actionable(filter) && is_actionable(k))
}

/// Human-readable name for a kind, driven by the TABLE.
pub fn long_name(k: Kind) -> &'static str {
    for s in TABLE {
        if s.kind == k {
            return s.long;
        }
    }
    unreachable!("every Kind variant appears in TABLE")
}

/// Parse a `--kind` token (short or long form from TABLE) into a Kind.
/// Pure table lookup — adding a marker row needs no change here.
/// `action`/`todo` are aliases (see `kind_matches_filter`).
pub fn parse_kind(token: &str) -> Option<Kind> {
    lookup(token).map(|s| s.kind)
}

/// Resolve a token (without `::`, any case) to its spec. Short or long form.
pub fn lookup(token: &str) -> Option<&'static MarkerSpec> {
    let t = token.to_ascii_lowercase();
    TABLE
        .iter()
        .find(|s| s.long == t || s.shorts.contains(&t.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_has_nine_kinds() {
        assert_eq!(TABLE.len(), 9);
    }

    // IN_TEST.md E1: short and long forms are equivalent.
    #[test]
    fn test_marker_long_form() {
        assert_eq!(lookup("h").unwrap().kind, lookup("hate").unwrap().kind);
        assert_eq!(lookup("?").unwrap().kind, lookup("question").unwrap().kind);
        assert_eq!(lookup("td").unwrap().kind, lookup("todo").unwrap().kind);
    }

    #[test]
    fn test_only_action_todo_tracked() {
        for s in TABLE {
            let want = matches!(s.kind, Kind::Action | Kind::Todo);
            assert_eq!(s.tracked, want, "{:?} tracked flag wrong", s.kind);
        }
    }

    #[test]
    fn test_parse_kind_action_aliases_todo() {
        let action = parse_kind("action").unwrap();
        assert!(kind_matches_filter(action, Kind::Action));
        assert!(kind_matches_filter(action, Kind::Todo));
        assert!(!kind_matches_filter(action, Kind::Hate));
        assert!(!kind_matches_filter(action, Kind::Note));
        // `todo` is an alias for `action`: same group either way.
        let todo = parse_kind("todo").unwrap();
        assert!(kind_matches_filter(todo, Kind::Action));
        assert!(kind_matches_filter(todo, Kind::Todo));
    }

    #[test]
    fn test_parse_kind_short_form() {
        let n = parse_kind("n").unwrap();
        assert!(kind_matches_filter(n, Kind::Note));
        assert!(!kind_matches_filter(n, Kind::Action));
    }

    #[test]
    fn test_parse_kind_unknown() {
        assert!(parse_kind("nonexistent").is_none());
    }

    #[test]
    fn test_long_name_all_kinds() {
        for s in TABLE {
            assert_eq!(long_name(s.kind), s.long, "long_name should match TABLE: {:?}", s.kind);
        }
    }
}

/// Generate a human-readable list of valid `--kind` values from TABLE.
/// Added here so adding a marker row updates help without touching CLI code.
pub fn kind_help_string() -> String {
    let rows = TABLE
        .iter()
        .map(|s| {
            let shorts = s.shorts.join(", ");
            format!("  {:<12} {}", s.long, shorts)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("Kinds (for --kind):\n{rows}")
}
