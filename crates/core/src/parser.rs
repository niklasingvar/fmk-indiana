// ::ignore
//! Line parser. Stateless per line except the declared fence state
//! (IN_PRINCIPLES.md: stateless per line). One indiana per line (IN_SCAN.md).
//!
//! A marker sits at column 0, or inline at the end of a *content* line — a
//! `::` preceded by whitespace, with at least one non-whitespace char earlier
//! on the line. A purely indented `::h` is therefore neither (not column 0,
//! no preceding content) and is ignored, satisfying both IN_SCAN.md (no
//! paragraph tracking needed) and IN_TEST.md E2 (`    ::h` is not a marker).
//! The whitespace requirement keeps glued `::` inert — `std::fs`,
//! `Kind::Action`, `x::f(y)` are path separators, not commands — which is
//! what lets the scan cover code files (IN_SCAN.md: all files).

use crate::agents::AgentCatalog;
use crate::markers::{self, Kind, Msg};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Auto-run claimed this marker; an agent turn is in flight (IN_AUTORUN.md).
    Working,
    Done,
    Failed,
}

impl Status {
    fn parse(s: &str) -> Option<Status> {
        match s {
            "working" => Some(Status::Working),
            "done" => Some(Status::Done),
            "failed" => Some(Status::Failed),
            _ => None,
        }
    }
}

/// A parsed marker. `column` is the byte offset of `::`, kept for scope (M8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub kind: Kind,
    pub raw_token: String,
    pub message: Option<String>,
    /// Numeric batch label (`-1`, `-2`, …), stripped from the message.
    pub group: Option<u64>,
    /// Named agent persona (`-m` / `-mike`), stripped from the message.
    /// Canonical agent name. Mutually exclusive with `group`.
    pub agent: Option<String>,
    pub id: Option<String>,
    pub status: Option<Status>,
    pub column: usize,
    /// The `-a` / `--auto` flag was present on an auto-runnable kind
    /// (IN_AUTORUN.md). Only Fix/Elaborate/Prompt ever set this; on other
    /// kinds a leading `-a` stays in the message, unchanged.
    pub auto: bool,
}

/// Outcome of parsing one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineResult {
    /// No marker on this line (or the line is inside a fence / frontmatter).
    None,
    /// Exactly one marker found.
    Marker(Marker),
    /// Two or more markers on the line — skipped, caller warns (IN_SCAN.md).
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fm {
    NotStarted,
    Open,
    Done,
}

const FRONTMATTER_MARKER_PREFIX: &str = "# frontmatter.";

/// Comment leaders recognized on a first-line `::ignore` (eslint-style opt-out
/// for files without frontmatter). A trailing `-->` / `*/` closer is allowed.
const IGNORE_COMMENT_LEADERS: &[&str] = &["//", "#", "<!--", "/*", "--", ";"];

/// True when the file opts out of scanning entirely (IN_SCAN.md). Two forms:
/// leading YAML frontmatter containing a column-zero `::ignore` line (or the
/// YAML-safe comment `# ::ignore`), or — for files without frontmatter — a
/// first line that is `::ignore` bare or behind a comment leader
/// (`// ::ignore`, `<!-- ::ignore -->`, …). Checked once per file, before
/// line parsing; an ignored file contributes no markers and no warnings.
/// Indented occurrences inside frontmatter (block scalars quoting the
/// directive) stay inert, same as every other frontmatter line.
pub fn file_ignored(text: &str) -> bool {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return false;
    };
    if first.trim() == "---" {
        for line in lines {
            if line.trim() == "---" {
                return false;
            }
            let t = line.trim_end();
            if t == "::ignore" || t == "# ::ignore" {
                return true;
            }
        }
        return false;
    }
    first_line_ignore(first)
}

/// `::ignore` on the file's first line, bare or behind one comment leader.
fn first_line_ignore(line: &str) -> bool {
    let t = line.trim();
    if t == "::ignore" {
        return true;
    }
    for leader in IGNORE_COMMENT_LEADERS {
        if let Some(rest) = t.strip_prefix(leader) {
            let rest = rest
                .trim()
                .trim_end_matches("-->")
                .trim_end_matches("*/")
                .trim_end();
            return rest == "::ignore";
        }
    }
    false
}

/// The one cross-line bit: independent ``` and ~~~ fences, plus leading
/// YAML frontmatter (IN_SCAN.md code fences). `line_no` is 1-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceState {
    backtick: bool,
    tilde: bool,
    fm: Fm,
    line_no: usize,
}

impl Default for FenceState {
    fn default() -> Self {
        FenceState {
            backtick: false,
            tilde: false,
            fm: Fm::NotStarted,
            line_no: 0,
        }
    }
}

impl FenceState {
    /// True if an opened fence was never closed at EOF — caller warns
    /// (IN_SCAN.md: warn on EOF inside an open fence).
    pub fn unclosed_at_eof(&self) -> bool {
        self.backtick || self.tilde
    }

    fn in_fence(&self) -> bool {
        self.backtick || self.tilde || self.fm == Fm::Open
    }
}

/// Parse one line with no known agents. Pure given (line, prior state).
pub fn parse_line(line: &str, st: &mut FenceState) -> LineResult {
    parse_line_with(line, st, &AgentCatalog::default())
}

/// Parse one line, advancing fence state. Pure given (line, prior state,
/// agents). `agents` supplies the known `-<name>` flag tokens for this root;
/// an empty catalog leaves any dash token in the message, unchanged.
pub fn parse_line_with(line: &str, st: &mut FenceState, agents: &AgentCatalog) -> LineResult {
    st.line_no += 1;
    let trimmed = line.trim_start();

    // YAML frontmatter: a leading `---` block at file start only (line 1),
    // closed by the next `---`. The only `---` special case (IN_SCAN.md).
    match st.fm {
        Fm::NotStarted => {
            if st.line_no == 1 && trimmed.trim_end() == "---" {
                st.fm = Fm::Open;
                return LineResult::None;
            }
            st.fm = Fm::Done; // no frontmatter; only line 1 can start it.
        }
        Fm::Open => {
            if trimmed.trim_end() == "---" {
                st.fm = Fm::Done;
                return LineResult::None;
            }
            // Property comments are the one explicit exception: column-zero
            // `# frontmatter.<key> ::...` remains valid YAML while every value,
            // ordinary comment, and indented scalar stays inert.
            if line.starts_with(FRONTMATTER_MARKER_PREFIX) {
                return scan_markers(line, agents);
            }
            return LineResult::None;
        }
        Fm::Done => {}
    }

    // Fence delimiters toggle independently; the delimiter line bears no marker.
    if trimmed.starts_with("```") {
        st.backtick = !st.backtick;
        return LineResult::None;
    }
    if trimmed.starts_with("~~~") {
        st.tilde = !st.tilde;
        return LineResult::None;
    }
    if st.in_fence() {
        return LineResult::None; // sample text inside a fence — never triggers.
    }

    scan_markers(line, agents)
}

/// Byte offset of the one marker's `::` on a content line, `None` when the
/// line bears no marker (or an ambiguous pair). The write chokepoint uses
/// this to edit the exact `::` the scan found — never a glued path separator
/// like `std::fs` earlier on the same line.
pub fn marker_column(line: &str) -> Option<usize> {
    match scan_markers(line, &AgentCatalog::default()) {
        LineResult::Marker(m) => Some(m.column),
        _ => None,
    }
}

/// Find marker candidates on a content line. >1 valid → ambiguous.
///
/// A `::` inside an inline code span (backtick-delimited) is ignored, by the
/// same rule as fenced code: quoted `::` is sample text, not a command
/// (IN_SCAN.md). Code spans follow the CommonMark rule — an opener of N
/// backticks is closed only by the next run of exactly N — so a span may
/// itself contain backtick runs (e.g. a triple ``` shown inline).
fn scan_markers(line: &str, agents: &AgentCatalog) -> LineResult {
    let bytes = line.as_bytes();
    let mut found: Vec<Marker> = Vec::new();

    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let start = i;
            while i < bytes.len() && bytes[i] == b'`' {
                i += 1;
            }
            // A matched span skips its content; an unmatched run is literal.
            if let Some(close) = find_closing_run(bytes, i, i - start) {
                i = close;
            }
        } else if bytes[i] == b':' && i + 1 < bytes.len() && bytes[i + 1] == b':' {
            // Position rule: column 0, or inline — whitespace immediately
            // before the `::` and content somewhere earlier on the line.
            // A glued `::` (`std::fs`, `Kind::Action`) is a path separator.
            let valid_pos = i == 0
                || (bytes[i - 1].is_ascii_whitespace()
                    && line[..i].chars().any(|c| !c.is_whitespace()));
            if valid_pos {
                if let Some(m) = parse_candidate(line, i, agents) {
                    found.push(m);
                }
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    match found.len() {
        0 => LineResult::None,
        1 => LineResult::Marker(found.pop().unwrap()),
        _ => LineResult::Ambiguous,
    }
}

/// Index just past the next backtick run of exactly `n`, scanning from `from`.
/// `None` if there is no such run (the opener was unmatched → literal).
fn find_closing_run(bytes: &[u8], from: usize, n: usize) -> Option<usize> {
    let mut j = from;
    while j < bytes.len() {
        if bytes[j] == b'`' {
            let start = j;
            while j < bytes.len() && bytes[j] == b'`' {
                j += 1;
            }
            if j - start == n {
                return Some(j);
            }
        } else {
            j += 1;
        }
    }
    None
}

/// Parse a `::` at byte index `at`. Returns a Marker only if the token is a
/// known kind. Strips an optional `[id]` / `[id:status]` bracket before the
/// message (IN_LINE.md: bracket is stripped before parsing).
fn parse_candidate(line: &str, at: usize, agents: &AgentCatalog) -> Option<Marker> {
    let after = &line[at + 2..];

    // Token: `?` or a run of ascii letters.
    let (token, rest) = if let Some(r) = after.strip_prefix('?') {
        ("?", r)
    } else {
        let end = after
            .find(|c: char| !c.is_ascii_alphabetic())
            .unwrap_or(after.len());
        (&after[..end], &after[end..])
    };
    if token.is_empty() {
        return None;
    }
    let spec = markers::lookup(token)?;

    // Optional bracket immediately after the token: `[id]` or `[id:status]`.
    let (mut id, mut status, mut rest) = (None, None, rest);
    if let Some(stripped) = rest.strip_prefix('[') {
        if let Some(close) = stripped.find(']') {
            let inner = &stripped[..close];
            let (id_part, status_part) = match inner.split_once(':') {
                Some((a, b)) => (a, Some(b)),
                None => (inner, None),
            };
            if !id_part.is_empty() {
                id = Some(id_part.to_string());
            }
            status = status_part.and_then(Status::parse);
            rest = &rest[close + 2..]; // past the `]`
        }
    }

    // Flags after the bracket, before the message. Numeric labels group markers
    // for manual batch copy/run; `-<name>` (or a unique first letter) tags a
    // named agent persona — the two are mutually exclusive. `-a` / `--auto`
    // remains restricted to auto-runnable kinds. Unknown or duplicate flags
    // stop the scan and become ordinary message text.
    let mut auto = false;
    let mut group = None;
    let mut agent: Option<String> = None;
    let mut scan = rest.trim_start();
    loop {
        let end = scan.find(char::is_whitespace).unwrap_or(scan.len());
        let flag = &scan[..end];
        if matches!(flag, "-a" | "--auto") && markers::is_auto_runnable(spec.kind) && !auto {
            auto = true;
            scan = scan[end..].trim_start();
            continue;
        }
        if group.is_none() && agent.is_none() {
            if let Some(number) = flag
                .strip_prefix('-')
                .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|digits| digits.parse::<u64>().ok())
                .filter(|number| *number > 0)
            {
                group = Some(number);
                scan = scan[end..].trim_start();
                continue;
            }
            if let Some(name) = flag
                .strip_prefix('-')
                .filter(|token| !token.is_empty())
                .and_then(|token| agents.resolve_flag(token))
            {
                agent = Some(name.to_string());
                scan = scan[end..].trim_start();
                continue;
            }
        }
        break;
    }
    rest = scan;

    // Message: remainder to end of line, trimmed. Only kinds that take one keep it.
    let msg_text = rest.trim();
    let message = match spec.msg {
        Msg::None => None,
        _ if msg_text.is_empty() => None,
        _ => Some(msg_text.to_string()),
    };

    Some(Marker {
        kind: spec.kind,
        raw_token: format!("::{token}"),
        message,
        group,
        agent,
        id,
        status,
        column: at,
        auto,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(line: &str) -> LineResult {
        parse_line(line, &mut FenceState::default())
    }

    fn marker(line: &str) -> Marker {
        match parse(line) {
            LineResult::Marker(m) => m,
            other => panic!("expected marker, got {other:?}"),
        }
    }

    // --- E1: marker detection ---

    #[test]
    fn test_marker_column_zero() {
        assert_eq!(marker("::h").kind, Kind::Hate);
    }

    #[test]
    fn test_marker_inline() {
        let m = marker("Some text ::l");
        assert_eq!(m.kind, Kind::Love);
        assert_eq!(m.column, 10);
    }

    #[test]
    fn test_marker_with_message() {
        let m = marker("::fix rename this");
        assert_eq!(m.kind, Kind::Fix);
        assert_eq!(m.message.as_deref(), Some("rename this"));
    }

    #[test]
    fn test_marker_bracket_stripped() {
        let m = marker("::action[bear-mouse] do it");
        assert_eq!(m.kind, Kind::Action);
        assert_eq!(m.message.as_deref(), Some("do it"));
        assert_eq!(m.id.as_deref(), Some("bear-mouse"));
        assert_eq!(m.status, None);
    }

    #[test]
    fn test_status_done() {
        let m = marker("::action[happy-otter:done] buy milk");
        assert_eq!(m.status, Some(Status::Done));
        assert_eq!(m.id.as_deref(), Some("happy-otter"));
        assert_eq!(m.message.as_deref(), Some("buy milk"));
    }

    // --- auto-run flag + working status (IN_AUTORUN.md) ---

    #[test]
    fn test_auto_flag_short() {
        let m = marker("::fix -a banana");
        assert_eq!(m.kind, Kind::Fix);
        assert!(m.auto);
        assert_eq!(
            m.message.as_deref(),
            Some("banana"),
            "-a stripped from message"
        );
    }

    #[test]
    fn test_auto_flag_long() {
        let m = marker("::elaborate --auto expand this");
        assert!(m.auto);
        assert_eq!(m.message.as_deref(), Some("expand this"));
    }

    #[test]
    fn test_auto_flag_no_message() {
        let m = marker("::fix -a");
        assert!(m.auto);
        assert_eq!(m.message, None);
    }

    #[test]
    fn test_auto_flag_on_prompt() {
        let m = marker("::prompt -a run the thing");
        assert_eq!(m.kind, Kind::Prompt);
        assert!(m.auto);
        assert_eq!(m.message.as_deref(), Some("run the thing"));
    }

    #[test]
    fn test_no_auto_flag_default_false() {
        let m = marker("::fix rename this");
        assert!(!m.auto);
        assert_eq!(m.message.as_deref(), Some("rename this"));
    }

    #[test]
    fn test_auto_flag_ignored_on_non_directive() {
        // `-a` on a kind that never auto-runs stays in the message untouched.
        let m = marker("::note -a is literal here");
        assert!(!m.auto);
        assert_eq!(m.message.as_deref(), Some("-a is literal here"));
    }

    #[test]
    fn test_unknown_dash_token_stops_flag_scan() {
        // `-x` is not a known flag → it (and the rest) is the message.
        let m = marker("::fix -x keep this");
        assert!(!m.auto);
        assert_eq!(m.message.as_deref(), Some("-x keep this"));
    }

    #[test]
    fn test_auto_flag_with_bracket() {
        // A claimed line: bracket present, no -a (already consumed). Working parses.
        let m = marker("::fix[happy-otter:working] banana");
        assert_eq!(m.status, Some(Status::Working));
        assert_eq!(m.id.as_deref(), Some("happy-otter"));
        assert_eq!(m.message.as_deref(), Some("banana"));
        assert!(!m.auto, "claimed line no longer carries the flag");
    }

    #[test]
    fn test_numeric_group_flag() {
        let m = marker("::fix -1 tighten this");
        assert_eq!(m.group, Some(1));
        assert_eq!(m.message.as_deref(), Some("tighten this"));
        assert!(!m.auto);
    }

    #[test]
    fn test_numeric_groups_support_multiple_labels() {
        assert_eq!(marker("::fix -2 first").group, Some(2));
        assert_eq!(marker("::note -42 second").group, Some(42));
    }

    #[test]
    fn test_numeric_group_coexists_with_auto_in_either_order() {
        for line in ["::fix -7 -a banana", "::fix --auto -7 banana"] {
            let m = marker(line);
            assert_eq!(m.group, Some(7), "{line}");
            assert!(m.auto, "{line}");
            assert_eq!(m.message.as_deref(), Some("banana"), "{line}");
        }
    }

    // --- agent persona flag (mutually exclusive with numeric groups) ---

    fn catalog() -> AgentCatalog {
        AgentCatalog {
            names: vec!["lisa".to_string(), "mike".to_string()],
        }
    }

    fn marker_with_agents(line: &str) -> Marker {
        match parse_line_with(line, &mut FenceState::default(), &catalog()) {
            LineResult::Marker(m) => m,
            other => panic!("expected marker, got {other:?}"),
        }
    }

    #[test]
    fn test_agent_flag_letter_and_full_name() {
        for line in ["::fix -m create this task", "::fix -mike create this task"] {
            let m = marker_with_agents(line);
            assert_eq!(m.agent.as_deref(), Some("mike"), "{line}");
            assert_eq!(m.group, None, "{line}");
            assert_eq!(m.message.as_deref(), Some("create this task"), "{line}");
        }
        let m = marker_with_agents("::note -l model the domain");
        assert_eq!(m.agent.as_deref(), Some("lisa"));
    }

    #[test]
    fn test_agent_flag_unknown_without_catalog_stays_message() {
        // No agents known (plain parse_line) → `-m` is ordinary message text.
        let m = marker("::fix -m create this task");
        assert_eq!(m.agent, None);
        assert_eq!(m.message.as_deref(), Some("-m create this task"));
    }

    #[test]
    fn test_agent_and_group_flags_are_mutually_exclusive() {
        // Numeric first: `-m` stops the flag scan and stays in the message.
        let m = marker_with_agents("::fix -1 -m polish the schema");
        assert_eq!(m.group, Some(1));
        assert_eq!(m.agent, None);
        assert_eq!(m.message.as_deref(), Some("-m polish the schema"));
        // Agent first: `-1` stops the flag scan and stays in the message.
        let m = marker_with_agents("::fix -m -1 polish the schema");
        assert_eq!(m.agent.as_deref(), Some("mike"));
        assert_eq!(m.group, None);
        assert_eq!(m.message.as_deref(), Some("-1 polish the schema"));
    }

    #[test]
    fn test_agent_flag_coexists_with_auto() {
        let m = marker_with_agents("::fix -a -m banana");
        assert!(m.auto);
        assert_eq!(m.agent.as_deref(), Some("mike"));
        assert_eq!(m.message.as_deref(), Some("banana"));
    }

    #[test]
    fn test_duplicate_agent_flag_stays_message() {
        let m = marker_with_agents("::fix -m -l banana");
        assert_eq!(m.agent.as_deref(), Some("mike"));
        assert_eq!(m.message.as_deref(), Some("-l banana"));
    }

    #[test]
    fn test_zero_is_not_a_group_flag() {
        let m = marker("::fix -0 keep literal");
        assert_eq!(m.group, None);
        assert_eq!(m.message.as_deref(), Some("-0 keep literal"));
    }

    // --- inline position rule: whitespace before `::` (IN_SCAN.md) ---

    #[test]
    fn test_glued_double_colon_is_not_a_marker() {
        // Path separators in code (and prose) never trigger.
        assert_eq!(parse("use std::fs;"), LineResult::None);
        assert_eq!(parse("let k = Kind::Action;"), LineResult::None);
        assert_eq!(parse("let x = path::f(y);"), LineResult::None);
        assert_eq!(parse("std::f32::MAX"), LineResult::None);
        assert_eq!(parse("see ns::action for details"), LineResult::None);
    }

    #[test]
    fn test_marker_after_glued_path_still_counts() {
        let m = marker("use std::fs; // ::fix drop this import");
        assert_eq!(m.kind, Kind::Fix);
        assert_eq!(m.message.as_deref(), Some("drop this import"));
    }

    #[test]
    fn test_marker_column_helper() {
        assert_eq!(marker_column("::h"), Some(0));
        assert_eq!(marker_column("use std::fs; // ::fix x"), Some(16));
        assert_eq!(marker_column("use std::fs;"), None);
        assert_eq!(marker_column("::h ::l"), None);
    }

    #[test]
    fn test_marker_ambiguous_line() {
        assert_eq!(parse("::h ::l"), LineResult::Ambiguous);
    }

    #[test]
    fn test_reaction_takes_no_message() {
        // Trailing text on a pure reaction is not captured as a message.
        assert_eq!(marker("::h whatever").message, None);
    }

    // --- E2: code fences ---

    fn parse_all(text: &str) -> Vec<Marker> {
        let mut st = FenceState::default();
        text.lines()
            .filter_map(|l| match parse_line(l, &mut st) {
                LineResult::Marker(m) => Some(m),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn test_fence_backtick() {
        assert!(parse_all("```\n::h\n```\n").is_empty());
    }

    #[test]
    fn test_fence_tilde() {
        assert!(parse_all("~~~\n::h\n~~~\n").is_empty());
    }

    #[test]
    fn test_fence_independent() {
        // ``` opens, ~~~ inside flips tilde on, ``` closes backtick,
        // tilde still open → marker after is ignored.
        assert!(parse_all("```\n~~~\n```\n::h\n").is_empty());
    }

    #[test]
    fn test_fence_unclosed() {
        let mut st = FenceState::default();
        for l in "```\n::h\n".lines() {
            parse_line(l, &mut st);
        }
        assert!(st.unclosed_at_eof());
        assert!(parse_all("```\n::h\n").is_empty());
    }

    #[test]
    fn test_fence_yaml_frontmatter() {
        assert!(parse_all("---\n::h\n---\nreal text\n").is_empty());
    }

    #[test]
    fn test_frontmatter_property_comment_marker() {
        let ms =
            parse_all("---\nstatus: draft\n# frontmatter.status ::fix change to approved\n---\n");
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].kind, Kind::Fix);
        assert_eq!(ms[0].message.as_deref(), Some("change to approved"));
    }

    #[test]
    fn test_frontmatter_ordinary_comments_and_values_stay_ignored() {
        assert!(parse_all(
            "---\nstatus: draft # ::fix ignored\n# note ::fix ignored\n  # frontmatter.status ::fix ignored\n---\n"
        )
        .is_empty());
    }

    // --- file-level ::ignore opt-out (IN_SCAN.md) ---

    #[test]
    fn test_file_ignored_frontmatter() {
        assert!(file_ignored("---\nstatus: draft\n::ignore\n---\n::h\n"));
        assert!(file_ignored("---\n# ::ignore\n---\nbody ::fix x\n"));
    }

    #[test]
    fn test_file_ignored_first_line_comment() {
        assert!(file_ignored("::ignore\nbody\n"));
        assert!(file_ignored("// ::ignore\nfn main() {}\n"));
        assert!(file_ignored("# ::ignore\nimport os\n"));
        assert!(file_ignored("<!-- ::ignore -->\n<p>hi</p>\n"));
        assert!(file_ignored("/* ::ignore */\nbody\n"));
    }

    #[test]
    fn test_file_ignored_negative_cases() {
        assert!(!file_ignored(""));
        assert!(!file_ignored("body\n::h\n"));
        // Only the first line (or frontmatter) can opt out.
        assert!(!file_ignored("body\n// ::ignore\n"));
        // After the closing `---` the directive is ordinary text.
        assert!(!file_ignored("---\nstatus: draft\n---\n::ignore\n"));
        // Indented inside frontmatter (quoted in a block scalar) stays inert.
        assert!(!file_ignored("---\nnote: |\n  ::ignore\n---\n"));
        // The directive must be the whole comment, not a substring.
        assert!(!file_ignored("# how ::ignore works\nbody\n"));
    }

    #[test]
    fn test_ignore_token_is_not_a_marker() {
        // `ignore` is a directive, not a marker kind — inert in prose.
        assert_eq!(parse("::ignore"), LineResult::None);
    }

    #[test]
    fn test_yaml_only_at_file_start() {
        // A `---` thematic break mid-document does not start frontmatter.
        let ms = parse_all("intro\n---\n::h\n");
        assert_eq!(ms.len(), 1);
        assert_eq!(ms[0].kind, Kind::Hate);
    }

    #[test]
    fn test_indented_ignored() {
        // `    ::h` — not column 0, no preceding content → not a marker.
        assert_eq!(parse("    ::h"), LineResult::None);
    }

    #[test]
    fn test_inline_code_ignored() {
        // `::` quoted in an inline code span is sample text, not a command.
        assert_eq!(parse("the token `::hate` means dislike"), LineResult::None);
        assert_eq!(
            parse("- `::action[happy-otter] buy milk`."),
            LineResult::None
        );
        // Two quoted markers in a table cell → not ambiguous, just ignored.
        assert_eq!(parse("| `::q`, `::?` | question |"), LineResult::None);
    }

    #[test]
    fn test_inline_code_span_with_backtick_run() {
        // A code span may contain a triple ``` shown inline; the `::h` after
        // it is still inside its own span → ignored (the IN_TEST.md:34 case).
        let line = "`test` — ` ``` ` opens, file has `::h` at end";
        assert_eq!(parse(line), LineResult::None);
    }

    #[test]
    fn test_unmatched_backtick_is_literal() {
        // A lone unmatched backtick is literal text; a real marker after it counts.
        let m = marker("a ` lonely tick then ::f go");
        assert_eq!(m.kind, Kind::Fix);
    }

    #[test]
    fn test_marker_after_closed_code_span() {
        // A real marker outside a closed inline span is still detected.
        let m = marker("see `foo()` ::f rename it");
        assert_eq!(m.kind, Kind::Fix);
        assert_eq!(m.message.as_deref(), Some("rename it"));
    }

    // --- E3: stateless per line ---

    #[test]
    fn test_parse_line_pure() {
        let samples = [
            "::h",
            "  ::l",
            "text ::f msg",
            "```",
            "::action[x:done] y",
            "no marker here",
            "::q",
            "::h ::l",
            "::n note ::fix two",
        ];
        for s in samples {
            let mut a = FenceState::default();
            let mut b = a.clone();
            let ra = parse_line(s, &mut a);
            let rb = parse_line(s, &mut b);
            assert_eq!(ra, rb, "impure for {s:?}");
            assert_eq!(a, b, "state diverged for {s:?}");
        }
    }

    #[test]
    fn test_one_marker_per_line() {
        let text: String = (0..50).map(|_| "::h\n").collect();
        assert_eq!(parse_all(&text).len(), 50);
    }
}
