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

/// Filter set for `--kind`: one logical group of marker kinds.
/// Backed by TABLE tokens; parsing is table-driven, not hand-enumerated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KindFilter {
    Action,
    Note,
    Question,
    Hate,
    Love,
    Keep,
    Fix,
    Elaborate,
}

impl KindFilter {
    /// Does this filter accept the given kind?
    pub fn matches(self, k: Kind) -> bool {
        match self {
            KindFilter::Action => matches!(k, Kind::Action | Kind::Todo),
            KindFilter::Note => matches!(k, Kind::Note),
            KindFilter::Question => matches!(k, Kind::Question),
            KindFilter::Hate => matches!(k, Kind::Hate),
            KindFilter::Love => matches!(k, Kind::Love),
            KindFilter::Keep => matches!(k, Kind::Keep),
            KindFilter::Fix => matches!(k, Kind::Fix),
            KindFilter::Elaborate => matches!(k, Kind::Elaborate),
        }
    }
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

/// Parse a `--kind` token (short or long form from TABLE) into a KindFilter.
/// Special-cases: `action` maps to Action+Todo.
pub fn parse_kind_filter(token: &str) -> Option<KindFilter> {
    let spec = lookup(token)?;
    let f = match spec.kind {
        Kind::Action => KindFilter::Action,
        Kind::Todo => KindFilter::Action,
        Kind::Note => KindFilter::Note,
        Kind::Question => KindFilter::Question,
        Kind::Hate => KindFilter::Hate,
        Kind::Love => KindFilter::Love,
        Kind::Keep => KindFilter::Keep,
        Kind::Fix => KindFilter::Fix,
        Kind::Elaborate => KindFilter::Elaborate,
    };
    Some(f)
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
    fn test_parse_kind_filter_action() {
        let f = parse_kind_filter("action").unwrap();
        assert!(f.matches(Kind::Action));
        assert!(f.matches(Kind::Todo));
        assert!(!f.matches(Kind::Hate));
        assert!(!f.matches(Kind::Note));
    }

    #[test]
    fn test_parse_kind_filter_short_form() {
        let f = parse_kind_filter("n").unwrap();
        assert!(f.matches(Kind::Note));
    }

    #[test]
    fn test_parse_kind_filter_unknown() {
        assert!(parse_kind_filter("nonexistent").is_none());
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
    TABLE
        .iter()
        .map(|s| {
            let shorts = s.shorts.join(", ");
            format!("  {:<12} {}", s.long, shorts)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
