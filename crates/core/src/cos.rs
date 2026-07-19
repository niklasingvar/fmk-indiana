//! Chief-of-staff store — `tasks.md` (Agent/Human queues) plus `log.md`
//! (append-only run record) under `.indiana/chief-of-staff/` (COS_PRD.md).
//!
//! tasks.md is user data: humans edit it freely. Machine writes touch only the
//! target line or append; every other byte — including lines that don't parse —
//! survives a rewrite untouched. log.md is machine-written and truncatable.

use crate::index::Index;
use crate::markers::{long_name, queue_of};
use crate::markers::Queue;
use crate::parser::Status;
use crate::write::{atomic_write, WriteResult};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn tracker_path(root: &Path) -> PathBuf {
    root.join(".indiana").join("chief-of-staff").join("tasks.md")
}

pub fn log_path(root: &Path) -> PathBuf {
    root.join(".indiana").join("chief-of-staff").join("log.md")
}

/// Seed skeletons. `crates/core/templates/` is the single authoring source
/// (MENTAL_MODEL.md); `append_*` also writes these when a file is missing so
/// capture works in un-refreshed roots too.
pub const TASKS_SEED: &str = include_str!("../templates/chief-of-staff/tasks.md");
pub const LOG_SEED: &str = include_str!("../templates/chief-of-staff/log.md");

/// Task lifecycle, one checkbox glyph per state (COS_PRD.md line grammar).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Open,
    Working,
    Done,
    Failed,
}

impl TaskState {
    pub fn glyph(self) -> char {
        match self {
            TaskState::Open => ' ',
            TaskState::Working => '>',
            TaskState::Done => 'x',
            TaskState::Failed => '!',
        }
    }

    pub fn from_glyph(c: char) -> Option<Self> {
        match c {
            ' ' => Some(TaskState::Open),
            '>' => Some(TaskState::Working),
            'x' => Some(TaskState::Done),
            '!' => Some(TaskState::Failed),
            _ => None,
        }
    }
}

/// Where a captured task came from: the marker's file and line at capture time.
/// A jump hint, not maintained truth — the id in source is the join key.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Origin {
    pub path: String,
    pub line: usize,
}

/// One tracker line.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub text: String,
    pub queue: Queue,
    pub state: TaskState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

/// One action-log line.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub ts: String,
    pub event: String,
    pub id: String,
    pub detail: String,
}

/// Parse the tracker. Missing file reads as empty; lines that don't match the
/// grammar (or sit outside a queue section) are simply not tasks.
pub fn load_tasks(root: &Path) -> Vec<Task> {
    let Ok(text) = fs::read_to_string(tracker_path(root)) else {
        return Vec::new();
    };
    let mut tasks = Vec::new();
    let mut queue = None;
    for line in text.lines() {
        if let Some(q) = section_queue(line) {
            queue = q;
            continue;
        }
        if let Some(task) = parse_task_line(line, queue) {
            tasks.push(task);
        }
    }
    tasks
}

/// Append a task to its queue section, creating the file from the seed when
/// missing. Mtime-guarded like write.rs; retries once internally.
pub fn append_task(root: &Path, task: &Task) -> io::Result<WriteResult> {
    let line = task_line(task);
    let queue = task.queue;
    rewrite_tracker(root, move |text| insert_in_section(text, queue, &line))
}

/// Flip one task's state glyph in place; every other byte survives.
pub fn set_task_state(root: &Path, id: &str, state: TaskState) -> io::Result<WriteResult> {
    let id = id.to_string();
    rewrite_tracker(root, move |text| set_state_in_text(text, &id, state))
}

/// Pure transform behind `set_task_state`, shared with the batched capture
/// pass: flip the first matching task line's glyph, byte-surgically.
fn set_state_in_text(text: &str, id: &str, state: TaskState) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut hit = false;
    for line in text.split_inclusive('\n') {
        if !hit {
            if let Some((_, line_id, _)) = split_task_line(line.trim_end_matches(['\n', '\r'])) {
                if line_id == id {
                    // `- [` is 3 bytes; the ASCII glyph is the byte after.
                    let mut fixed = line.to_string();
                    fixed.replace_range(3..4, &state.glyph().to_string());
                    out.push_str(&fixed);
                    hit = true;
                    continue;
                }
            }
        }
        out.push_str(line);
    }
    hit.then_some(out)
}

/// Current state lookup used by capture/reconcile and `task done` feedback.
pub fn find_task(root: &Path, id: &str) -> Option<Task> {
    load_tasks(root).into_iter().find(|t| t.id == id)
}

/// Append one event line to log.md (seeded when missing). Single O_APPEND
/// write; concurrent appenders interleave whole lines.
pub fn append_log(root: &Path, event: &str, id: &str, detail: &str) -> io::Result<()> {
    append_log_batch(
        root,
        &[LogLine {
            event: event.to_string(),
            id: id.to_string(),
            detail: detail.to_string(),
        }],
    )
}

struct LogLine {
    event: String,
    id: String,
    detail: String,
}

/// Append many event lines in one O_APPEND write (a capture burst is one
/// syscall, not N), seeding the file when missing.
fn append_log_batch(root: &Path, lines: &[LogLine]) -> io::Result<()> {
    if lines.is_empty() {
        return Ok(());
    }
    let path = log_path(root);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        atomic_write(&path, LOG_SEED.as_bytes())?;
    }
    let (date, time) = now_stamp();
    let mut buf = String::new();
    for l in lines {
        if l.detail.is_empty() {
            buf.push_str(&format!("{date} {time} {} [{}]\n", l.event, l.id));
        } else {
            buf.push_str(&format!("{date} {time} {} [{}] {}\n", l.event, l.id, l.detail));
        }
    }
    use std::io::Write as _;
    let mut f = fs::OpenOptions::new().append(true).open(&path)?;
    f.write_all(buf.as_bytes())
}

/// Parse log.md into entries, file order (oldest first). Missing file = empty.
pub fn load_log(root: &Path) -> Vec<LogEntry> {
    let Ok(text) = fs::read_to_string(log_path(root)) else {
        return Vec::new();
    };
    text.lines().filter_map(parse_log_line).collect()
}

/// Today's date, UTC — the `created` stamp for captured/added tasks.
pub fn today() -> String {
    now_stamp().0
}

/// Full UTC timestamp `YYYY-MM-DD HH:MM` — the same shape log lines carry.
pub fn now() -> String {
    let (date, time) = now_stamp();
    format!("{date} {time}")
}

/// What the capture pass wrote, so the daemon can suppress its own writes.
#[derive(Debug, Default)]
pub struct CaptureReport {
    pub written: Vec<PathBuf>,
}

/// The capture + reconcile pass, run only at deliberate entry points
/// (daemon rebuilds and explicit `indiana scan <path>` — ScanOptions.capture).
/// A whole pass is at most one tracker rewrite plus one log append — a burst
/// of new markers never fans out into N writes; a marker-free rebuild is
/// write-free.
///
/// - capture: a tracked marker whose id is not in tasks.md gets a tracker line
///   in its TABLE queue, with origin backlink and date, plus a `capture` log
///   line. Id presence — in the file or already planned this pass (a
///   copy-pasted line duplicates its bracket) — is the double-capture guard.
/// - reconcile: a captured task whose id vanished from the source is done —
///   marker removal is the Indiana convention for done (`resolved` log line).
///   A file that failed to read merely hides its markers; its tasks are left
///   alone. A live marker's `:working`/`:failed`/`:done` status is mirrored
///   onto the task. Bare markers never flip a task back: human edits stand.
/// - human-added tasks (no origin) are never touched.
pub fn capture_and_reconcile(root: &Path, index: &Index) -> CaptureReport {
    let mut report = CaptureReport::default();
    let tasks = load_tasks(root);
    let known: HashMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut live: HashMap<&str, Option<Status>> = HashMap::new();
    let mut appends: Vec<Task> = Vec::new();
    let mut log_lines: Vec<LogLine> = Vec::new();

    for m in &index.markers {
        let (Some(queue), Some(id)) = (queue_of(m.kind), m.id.as_deref()) else {
            continue;
        };
        live.insert(id, m.status);
        if known.contains_key(id) || appends.iter().any(|t| t.id == id) {
            continue;
        }
        let rel = m.path.strip_prefix(root).unwrap_or(&m.path);
        let task = Task {
            id: id.to_string(),
            text: m.message.clone().unwrap_or_default(),
            queue,
            state: task_state_of(m.status).unwrap_or(TaskState::Open),
            origin: Some(Origin {
                path: rel.display().to_string(),
                line: m.line,
            }),
            created: Some(today()),
        };
        log_lines.push(LogLine {
            event: "capture".to_string(),
            id: id.to_string(),
            detail: format!("{} {}:{} - {}", long_name(m.kind), rel.display(), m.line, task.text),
        });
        appends.push(task);
    }

    // Files that failed to read this scan hide their markers — that is not
    // resolution. (Deleting or gitignoring a file still resolves its tasks:
    // indistinguishable from marker removal; documented tradeoff, COS_PRD.md.)
    let unreadable: Vec<PathBuf> = index
        .warnings
        .iter()
        .filter_map(|w| w.split_once(": unreadable").map(|(p, _)| PathBuf::from(p)))
        .collect();

    let mut flips: Vec<(String, TaskState)> = Vec::new();
    for task in &tasks {
        let Some(origin) = &task.origin else {
            continue;
        };
        match live.get(task.id.as_str()) {
            None => {
                if unreadable.iter().any(|p| *p == root.join(&origin.path)) {
                    continue;
                }
                if task.state != TaskState::Done {
                    flips.push((task.id.clone(), TaskState::Done));
                    log_lines.push(LogLine {
                        event: "resolved".to_string(),
                        id: task.id.clone(),
                        detail: "marker removed from source".to_string(),
                    });
                }
            }
            Some(status) => {
                if let Some(state) = task_state_of(*status) {
                    if task.state != state {
                        flips.push((task.id.clone(), state));
                    }
                }
            }
        }
    }

    if appends.is_empty() && flips.is_empty() {
        return report;
    }
    let result = rewrite_tracker(root, |text| {
        let mut current = text.to_string();
        let mut changed = false;
        for (id, state) in &flips {
            if let Some(next) = set_state_in_text(&current, id, *state) {
                current = next;
                changed = true;
            }
        }
        for task in &appends {
            if current.contains(&format!("[{}]", task.id)) {
                continue; // a racing writer captured it meanwhile
            }
            if let Some(next) = insert_in_section(&current, task.queue, &task_line(task)) {
                current = next;
                changed = true;
            }
        }
        changed.then_some(current)
    });
    if matches!(result, Ok(WriteResult::Written)) {
        report.written.push(tracker_path(root));
        if !log_lines.is_empty() && append_log_batch(root, &log_lines).is_ok() {
            report.written.push(log_path(root));
        }
    }
    // A write lost twice to a racing writer logs nothing; the next scan
    // recomputes from source and reconverges.
    report
}

fn task_state_of(status: Option<Status>) -> Option<TaskState> {
    match status? {
        Status::Working => Some(TaskState::Working),
        Status::Done => Some(TaskState::Done),
        Status::Failed => Some(TaskState::Failed),
    }
}

// ── line grammar ────────────────────────────────────────────────────────────

fn section_queue(line: &str) -> Option<Option<Queue>> {
    let t = line.trim_end();
    if t.eq_ignore_ascii_case("## agent") {
        Some(Some(Queue::Agent))
    } else if t.eq_ignore_ascii_case("## human") {
        Some(Some(Queue::Human))
    } else if t.starts_with('#') {
        Some(None) // any other heading ends the queue section
    } else {
        None
    }
}

/// `- [<state>] [<id>] <rest>` → (state, id, rest). None when not a task line.
fn split_task_line(line: &str) -> Option<(TaskState, &str, &str)> {
    let rest = line.strip_prefix("- [")?;
    let mut chars = rest.chars();
    let state = TaskState::from_glyph(chars.next()?)?;
    let rest = chars.as_str().strip_prefix("] [")?;
    let close = rest.find(']')?;
    let id = &rest[..close];
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return None;
    }
    Some((state, id, rest[close + 1..].trim_start()))
}

fn parse_task_line(line: &str, queue: Option<Queue>) -> Option<Task> {
    let (state, id, rest) = split_task_line(line)?;
    let mut text = rest.trim_end();

    // Trailing `YYYY-MM-DD`, then trailing `(path:line)`, peeled from the end.
    let mut created = None;
    if let Some((head, tail)) = text.rsplit_once(' ') {
        if is_date(tail) {
            created = Some(tail.to_string());
            text = head.trim_end();
        }
    }
    let mut origin = None;
    if text.ends_with(')') {
        if let Some(open) = text.rfind('(') {
            if let Some((p, l)) = text[open + 1..text.len() - 1].rsplit_once(':') {
                if let (false, Ok(line_no)) = (p.contains(char::is_whitespace), l.parse()) {
                    origin = Some(Origin {
                        path: p.to_string(),
                        line: line_no,
                    });
                    text = text[..open].trim_end();
                }
            }
        }
    }

    Some(Task {
        id: id.to_string(),
        text: text.to_string(),
        queue: queue?,
        state,
        origin,
        created,
    })
}

/// Render a task back to its line. Round-trips with `parse_task_line`.
pub fn task_line(task: &Task) -> String {
    let mut line = format!("- [{}] [{}] {}", task.state.glyph(), task.id, task.text);
    if let Some(o) = &task.origin {
        line.push_str(&format!(" ({}:{})", o.path, o.line));
    }
    if let Some(d) = &task.created {
        line.push_str(&format!(" {d}"));
    }
    line
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b.iter()
            .enumerate()
            .all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // `YYYY-MM-DD HH:MM <event> [<id>] <detail…>`
    let (date, rest) = line.split_once(' ')?;
    if !is_date(date) {
        return None;
    }
    let (time, rest) = rest.split_once(' ')?;
    let (event, rest) = rest.split_once(" [")?;
    let (id, detail) = rest.split_once(']')?;
    Some(LogEntry {
        ts: format!("{date} {time}"),
        event: event.to_string(),
        id: id.to_string(),
        detail: detail.trim().to_string(),
    })
}

// ── tracker rewrite plumbing ────────────────────────────────────────────────

/// Read-modify-write tasks.md through the write.rs chokepoint discipline
/// (`guarded_rewrite`): mtime guard, atomic replace, one retry, seed on
/// missing. `f` returns the new full text, or None for a no-op.
fn rewrite_tracker<F>(root: &Path, f: F) -> io::Result<WriteResult>
where
    F: Fn(&str) -> Option<String>,
{
    crate::write::guarded_rewrite(&tracker_path(root), TASKS_SEED.as_bytes(), f)
}

/// Insert `line` at the end of `queue`'s section (before the next heading,
/// above trailing blanks). A missing section heading is appended at EOF —
/// hand-edited files degrade, they don't break.
fn insert_in_section(text: &str, queue: Queue, line: &str) -> Option<String> {
    let heading = match queue {
        Queue::Agent => "## Agent",
        Queue::Human => "## Human",
    };
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let head_idx = lines
        .iter()
        .position(|l| l.trim_end().eq_ignore_ascii_case(heading));

    let mut out = String::with_capacity(text.len() + line.len() + 1);
    match head_idx {
        Some(h) => {
            // Section ends at the next heading or EOF; insert above trailing blanks.
            let mut end = lines.len();
            for (i, l) in lines.iter().enumerate().skip(h + 1) {
                if l.trim_end().starts_with('#') {
                    end = i;
                    break;
                }
            }
            let mut insert_at = end;
            while insert_at > h + 1 && lines[insert_at - 1].trim().is_empty() {
                insert_at -= 1;
            }
            for (i, l) in lines.iter().enumerate() {
                if i == insert_at {
                    out.push_str(line);
                    out.push('\n');
                }
                out.push_str(l);
            }
            if insert_at == lines.len() {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(line);
                out.push('\n');
            }
        }
        None => {
            out.push_str(text);
            if !out.ends_with('\n') && !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("\n{heading}\n{line}\n"));
        }
    }
    Some(out)
}

// ── UTC timestamp without a date dependency ─────────────────────────────────

fn now_stamp() -> (String, String) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    (
        format!("{y:04}-{m:02}-{d:02}"),
        format!("{:02}:{:02}", rem / 3600, (rem % 3600) / 60),
    )
}

/// Days-since-epoch → (year, month, day). Howard Hinnant's civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let d = std::env::temp_dir().join(format!(
            "indiana-cos-{nanos}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn task(id: &str, queue: Queue) -> Task {
        Task {
            id: id.to_string(),
            text: format!("do the {id} thing"),
            queue,
            state: TaskState::Open,
            origin: Some(Origin {
                path: "docs/x.md".into(),
                line: 7,
            }),
            created: Some("2026-07-16".into()),
        }
    }

    #[test]
    fn test_task_line_round_trip() {
        for t in [
            task("beka-lun", Queue::Agent),
            Task {
                origin: None,
                created: None,
                state: TaskState::Failed,
                ..task("mira-tok", Queue::Human)
            },
            Task {
                text: "text with (parens:inside) kept".into(),
                origin: None,
                ..task("dopa-rin", Queue::Agent)
            },
        ] {
            let line = task_line(&t);
            let parsed = parse_task_line(&line, Some(t.queue)).unwrap();
            assert_eq!(parsed, t, "line: {line}");
        }
    }

    #[test]
    fn test_append_and_load_by_queue() {
        let d = tmp();
        append_task(&d, &task("beka-lun", Queue::Agent)).unwrap();
        append_task(&d, &task("suna-vel", Queue::Human)).unwrap();
        append_task(&d, &task("mira-tok", Queue::Agent)).unwrap();
        let tasks = load_tasks(&d);
        assert_eq!(tasks.len(), 3);
        assert_eq!(
            tasks
                .iter()
                .filter(|t| t.queue == Queue::Agent)
                .map(|t| t.id.as_str())
                .collect::<Vec<_>>(),
            ["beka-lun", "mira-tok"]
        );
        assert_eq!(tasks.iter().filter(|t| t.queue == Queue::Human).count(), 1);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_set_state_touches_one_byte() {
        let d = tmp();
        append_task(&d, &task("beka-lun", Queue::Agent)).unwrap();
        let before = fs::read_to_string(tracker_path(&d)).unwrap();
        set_task_state(&d, "beka-lun", TaskState::Done).unwrap();
        let after = fs::read_to_string(tracker_path(&d)).unwrap();
        assert_eq!(before.replace("- [ ] [beka-lun]", "- [x] [beka-lun]"), after);
        assert_eq!(find_task(&d, "beka-lun").unwrap().state, TaskState::Done);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_unknown_lines_survive_rewrites() {
        let d = tmp();
        append_task(&d, &task("beka-lun", Queue::Agent)).unwrap();
        // Hand edits: prose, a malformed task, a stray list item.
        let path = tracker_path(&d);
        let mut text = fs::read_to_string(&path).unwrap();
        text.push_str("\nsome prose note\n- [?] [not-a-state] nope\n- plain item\n");
        fs::write(&path, &text).unwrap();

        append_task(&d, &task("mira-tok", Queue::Human)).unwrap();
        set_task_state(&d, "beka-lun", TaskState::Working).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        for kept in ["some prose note", "- [?] [not-a-state] nope", "- plain item"] {
            assert!(after.contains(kept), "lost {kept:?} in:\n{after}");
        }
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_human_added_line_without_origin_or_date() {
        let d = tmp();
        let path = tracker_path(&d);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# Tasks\n\n## Human\n- [ ] [suna-vel] review the notes\n").unwrap();
        let tasks = load_tasks(&d);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].queue, Queue::Human);
        assert_eq!(tasks[0].text, "review the notes");
        assert_eq!(tasks[0].origin, None);
        assert_eq!(tasks[0].created, None);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_task_outside_section_is_not_a_task() {
        let d = tmp();
        let path = tracker_path(&d);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "- [ ] [beka-lun] floating\n\n## Agent\n").unwrap();
        assert!(load_tasks(&d).is_empty());
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_missing_section_heading_degrades() {
        let d = tmp();
        let path = tracker_path(&d);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "# Tasks\n\n## Agent\n").unwrap();
        append_task(&d, &task("suna-vel", Queue::Human)).unwrap();
        let tasks = load_tasks(&d);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].queue, Queue::Human);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_log_append_and_load() {
        let d = tmp();
        append_log(&d, "capture", "beka-lun", "todo docs/x.md:7 - do the thing").unwrap();
        append_log(&d, "done", "beka-lun", "").unwrap();
        let entries = load_log(&d);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].event, "capture");
        assert_eq!(entries[0].id, "beka-lun");
        assert!(entries[0].detail.starts_with("todo docs/x.md:7"));
        assert_eq!(entries[1].event, "done");
        assert_eq!(entries[1].detail, "");
        let text = fs::read_to_string(log_path(&d)).unwrap();
        assert!(text.starts_with("---\n"), "log carries frontmatter");
        fs::remove_dir_all(&d).ok();
    }

    // ── capture + reconcile over a real scan ───────────────────────────────

    fn write(dir: &Path, rel: &str, body: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    /// A capture-enabled build — what the daemon and `indiana scan <path>` run.
    fn build_capture(root: &Path) -> crate::index::ScanReport {
        crate::index::Index::build_with_options(
            root,
            crate::index::ScanOptions::write_ids().with_capture(),
        )
    }

    #[test]
    fn test_capture_routes_queues_and_logs() {
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n::task wire panel\n::action review notes\n");
        build_capture(&d);

        let tasks = load_tasks(&d);
        assert_eq!(tasks.len(), 3);
        let by_text = |s: &str| tasks.iter().find(|t| t.text == s).unwrap();
        assert_eq!(by_text("tighten help").queue, Queue::Agent);
        assert_eq!(by_text("wire panel").queue, Queue::Agent); // ::task aliases todo
        assert_eq!(by_text("review notes").queue, Queue::Human);
        let origin = by_text("tighten help").origin.as_ref().unwrap();
        assert_eq!(origin.path, "doc.md");
        assert_eq!(origin.line, 1);

        let log = load_log(&d);
        assert_eq!(log.iter().filter(|e| e.event == "capture").count(), 3);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_capture_idempotent_across_builds() {
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n");
        build_capture(&d);
        let tracker_after_first = fs::read_to_string(tracker_path(&d)).unwrap();
        let report = build_capture(&d);
        assert!(report.written_paths.is_empty(), "second build must be write-free");
        assert_eq!(tracker_after_first, fs::read_to_string(tracker_path(&d)).unwrap());
        assert_eq!(load_log(&d).len(), 1);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_marker_removed_resolves_task() {
        let d = tmp();
        write(&d, "doc.md", "keep\n::todo tighten help\n");
        build_capture(&d);
        write(&d, "doc.md", "keep\n");
        build_capture(&d);

        let task = &load_tasks(&d)[0];
        assert_eq!(task.state, TaskState::Done);
        assert!(load_log(&d).iter().any(|e| e.event == "resolved"));
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_marker_status_mirrors_onto_task() {
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n");
        build_capture(&d);
        let id = load_tasks(&d)[0].id.clone();
        write(&d, "doc.md", &format!("::todo[{id}:working] tighten help\n"));
        build_capture(&d);
        assert_eq!(load_tasks(&d)[0].state, TaskState::Working);

        write(&d, "doc.md", &format!("::todo[{id}:failed] tighten help\n"));
        build_capture(&d);
        assert_eq!(load_tasks(&d)[0].state, TaskState::Failed);
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_human_done_stands_while_bare_marker_lives() {
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n");
        build_capture(&d);
        let id = load_tasks(&d)[0].id.clone();
        set_task_state(&d, &id, TaskState::Done).unwrap();
        build_capture(&d);
        assert_eq!(load_tasks(&d)[0].state, TaskState::Done, "bare marker must not reopen");
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_human_added_task_untouched_by_reconcile() {
        let d = tmp();
        append_task(
            &d,
            &Task {
                id: "suna-vel".into(),
                text: "hand-added".into(),
                queue: Queue::Human,
                state: TaskState::Open,
                origin: None,
                created: None,
            },
        )
        .unwrap();
        build_capture(&d);
        let task = &load_tasks(&d)[0];
        assert_eq!(task.state, TaskState::Open, "no origin, never reconciled");
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_duplicate_id_captures_once() {
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n");
        build_capture(&d);
        let id = load_tasks(&d)[0].id.clone();
        // Copy-paste the claimed line (same bracket id) into another file.
        write(&d, "copy.md", &format!("::todo[{id}] tighten help\n"));
        build_capture(&d);
        let rows = load_tasks(&d);
        assert_eq!(rows.len(), 1, "one id, one row: {rows:?}");
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_capture_off_without_opt_in() {
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n");
        // A plain write scan injects ids but must not mint chief-of-staff files.
        crate::index::Index::build(&d);
        assert!(!tracker_path(&d).exists());
        assert!(!log_path(&d).exists());
        fs::remove_dir_all(&d).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_unreadable_file_does_not_resolve() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmp();
        write(&d, "doc.md", "::todo tighten help\n");
        build_capture(&d);
        assert_eq!(load_tasks(&d)[0].state, TaskState::Open);

        let doc = d.join("doc.md");
        fs::set_permissions(&doc, fs::Permissions::from_mode(0o000)).unwrap();
        build_capture(&d);
        fs::set_permissions(&doc, fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            load_tasks(&d)[0].state,
            TaskState::Open,
            "a hidden marker is not a resolved marker"
        );
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn test_civil_from_days() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_285), (2025, 7, 16)); // spot check
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
    }
}
