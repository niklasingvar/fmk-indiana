//! The one marker table (IN_PRINCIPLES.md: one table drives everything).
//! Parser, compiler, and identity all read this; none re-encode the set.
//! Adding a marker is one row here (IN_COMMANDS.md "The set").

/// What a marker means. One variant per row of the IN_COMMANDS.md table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
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
    MarkerSpec { shorts: &["q", "?"], long: "question",  kind: Kind::Question,  msg: Msg::Optional, tracked: false },
    MarkerSpec { shorts: &["h"],      long: "hate",      kind: Kind::Hate,      msg: Msg::None,     tracked: false },
    MarkerSpec { shorts: &["l"],      long: "love",      kind: Kind::Love,      msg: Msg::None,     tracked: false },
    MarkerSpec { shorts: &["k"],      long: "keep",      kind: Kind::Keep,      msg: Msg::None,     tracked: false },
    MarkerSpec { shorts: &["f"],      long: "fix",       kind: Kind::Fix,       msg: Msg::Optional, tracked: false },
    MarkerSpec { shorts: &["e"],      long: "elaborate", kind: Kind::Elaborate, msg: Msg::Optional, tracked: false },
    MarkerSpec { shorts: &["n"],      long: "note",      kind: Kind::Note,      msg: Msg::Required, tracked: false },
    MarkerSpec { shorts: &["a"],      long: "action",    kind: Kind::Action,    msg: Msg::Required, tracked: true  },
    MarkerSpec { shorts: &["td"],     long: "todo",      kind: Kind::Todo,      msg: Msg::Required, tracked: true  },
];

/// Resolve a token (without `::`, any case) to its spec. Short or long form.
pub fn lookup(token: &str) -> Option<&'static MarkerSpec> {
    let t = token.to_ascii_lowercase();
    TABLE.iter().find(|s| s.long == t || s.shorts.contains(&t.as_str()))
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
}
