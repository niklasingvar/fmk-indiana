//! Write chokepoint — the only core function that mutates user markdown.

use crate::id::IdGenerator;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteResult {
    Unchanged,
    Written,
    Retry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectRequest {
    pub path: PathBuf,
    pub line_no: usize,
}

pub fn inject(requests: &[InjectRequest], read_only: bool) -> HashMap<PathBuf, WriteResult> {
    let mut by_path: HashMap<PathBuf, BTreeSet<usize>> = HashMap::new();
    for req in requests {
        by_path
            .entry(req.path.clone())
            .or_default()
            .insert(req.line_no);
    }

    let mut results = HashMap::new();
    for (path, lines) in by_path {
        let result = if read_only {
            WriteResult::Unchanged
        } else {
            inject_file(&path, &lines).unwrap_or(WriteResult::Retry)
        };
        results.insert(path, result);
    }
    results
}

fn inject_file(path: &Path, target_lines: &BTreeSet<usize>) -> io::Result<WriteResult> {
    let before = match fs::metadata(path) {
        Ok(m) => m.modified().ok(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(WriteResult::Unchanged),
        Err(e) => return Err(e),
    };
    let bytes = fs::read(path)?;
    let ranges = line_ranges(&bytes);
    let mut ids = IdGenerator::new();

    for (start, end) in &ranges {
        if let Some((id, true)) = existing_bracket(&bytes[*start..*end]) {
            ids.register(&id);
        }
    }

    let mut changes = Vec::new();
    for line_no in target_lines {
        let Some((start, end)) = ranges.get(line_no.saturating_sub(1)).copied() else {
            continue;
        };
        if let Some(line) = normalize_line(&bytes[start..end], &mut ids) {
            changes.push((start, end, line));
        }
    }
    if changes.is_empty() {
        return Ok(WriteResult::Unchanged);
    }

    let after = fs::metadata(path)?.modified().ok();
    if before != after {
        return Ok(WriteResult::Retry);
    }

    let mut out = bytes;
    for (start, end, line) in changes.into_iter().rev() {
        out.splice(start..end, line);
    }
    atomic_write(path, &out)?;
    Ok(WriteResult::Written)
}

/// Set the status word on one marker line: mint an id if absent, write the
/// bracket as `[id:status]`, and strip a leading `-a`/`--auto` flag from the
/// message. The single write that moves an auto-run marker between states —
/// `working` on claim, `done`/`failed` on completion (IN_AUTORUN.md). Same
/// contract as `inject`: byte-preserving elsewhere, atomic, mtime-guarded, and
/// idempotent (a line already at the target state re-runs byte-identical).
pub fn set_status(path: &Path, line_no: usize, status: &str) -> io::Result<WriteResult> {
    debug_assert!(IdGenerator::is_valid_status(status));
    let before = match fs::metadata(path) {
        Ok(m) => m.modified().ok(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(WriteResult::Unchanged),
        Err(e) => return Err(e),
    };
    let bytes = fs::read(path)?;
    let ranges = line_ranges(&bytes);
    let mut ids = IdGenerator::new();
    for (start, end) in &ranges {
        if let Some((id, true)) = existing_bracket(&bytes[*start..*end]) {
            ids.register(&id);
        }
    }
    let Some((start, end)) = ranges.get(line_no.saturating_sub(1)).copied() else {
        return Ok(WriteResult::Unchanged);
    };
    let Some(new_line) = status_line(&bytes[start..end], status, &mut ids) else {
        return Ok(WriteResult::Unchanged); // no marker, or already byte-identical
    };

    let after = fs::metadata(path)?.modified().ok();
    if before != after {
        return Ok(WriteResult::Retry);
    }
    let mut out = bytes;
    out.splice(start..end, new_line);
    atomic_write(path, &out)?;
    Ok(WriteResult::Written)
}

/// Rewrite one line's bracket to `[id:status]` and strip auto flags. Returns
/// `None` if the line has no marker or the result equals the input (idempotent).
fn status_line(line: &[u8], status: &str, ids: &mut IdGenerator) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(line).ok()?;
    let at = text.find("::")?;
    let after = &text[at + 2..];
    let token_len = after
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(after.len());
    if token_len == 0 {
        return None;
    }
    let token_end = at + 2 + token_len;
    let rest = &text[token_end..];

    // Reuse an existing valid/repairable id; otherwise mint one. Capture the
    // message that follows any existing bracket.
    let (id, suffix) = match rest
        .strip_prefix('[')
        .and_then(|s| s.find(']').map(|c| (s, c)))
    {
        Some((stripped, close)) => {
            let (existing, _st, valid) = parse_bracket(&stripped[..close]);
            let id = if valid || IdGenerator::is_valid(&existing) {
                ids.register(&existing);
                existing
            } else {
                ids.next()
            };
            (id, &rest[close + 2..])
        }
        None => (ids.next(), rest),
    };

    let message = strip_leading_auto_flags(suffix);
    let new_line = format!("{}[{id}:{status}]{message}", &text[..token_end]);
    if new_line.as_bytes() == line {
        return None;
    }
    Some(new_line.into_bytes())
}

/// Drop `-a` / `--auto` from the recognized flag prefix while retaining one
/// numeric group label. This handles both `-a -1` and `-1 -a`.
fn strip_leading_auto_flags(s: &str) -> String {
    let mut rest = s;
    let mut group = None;
    loop {
        let trimmed = rest.trim_start_matches([' ', '\t']);
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let flag = &trimmed[..end];
        if matches!(flag, "-a" | "--auto") {
            rest = &trimmed[end..];
            continue;
        }
        if group.is_none()
            && flag
                .strip_prefix('-')
                .filter(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|digits| digits.parse::<u64>().ok())
                .is_some_and(|number| number > 0)
        {
            group = Some(flag);
            rest = &trimmed[end..];
            continue;
        }
        break;
    }
    match group {
        Some(group) => format!(" {group}{rest}"),
        None => rest.to_string(),
    }
}

fn normalize_line(line: &[u8], ids: &mut IdGenerator) -> Option<Vec<u8>> {
    let text = std::str::from_utf8(line).ok()?;
    let at = text.find("::")?;
    let after = &text[at + 2..];
    let token_len = after
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(after.len());
    if token_len == 0 {
        return None;
    }
    let token_end = at + 2 + token_len;
    let rest = &text[token_end..];

    if let Some(stripped) = rest.strip_prefix('[') {
        if let Some(close) = stripped.find(']') {
            let inner = &stripped[..close];
            let (id, status, valid) = parse_bracket(inner);
            if valid {
                ids.register(&id);
                return None;
            }

            let id = if IdGenerator::is_valid(&id) {
                ids.register(&id);
                id
            } else {
                ids.next()
            };
            let bracket = match status
                .as_deref()
                .filter(|s| IdGenerator::is_valid_status(s))
            {
                Some(status) => format!("[{id}:{status}]"),
                None => format!("[{id}]"),
            };
            let suffix = &rest[close + 2..];
            return Some(format!("{}{}{}", &text[..token_end], bracket, suffix).into_bytes());
        }
    }

    let id = ids.next();
    Some(format!("{}[{id}]{}", &text[..token_end], rest).into_bytes())
}

fn existing_bracket(line: &[u8]) -> Option<(String, bool)> {
    let text = std::str::from_utf8(line).ok()?;
    let at = text.find("::")?;
    let after = &text[at + 2..];
    let token_len = after
        .find(|c: char| !c.is_ascii_alphabetic())
        .unwrap_or(after.len());
    let rest = &after[token_len..];
    let stripped = rest.strip_prefix('[')?;
    let close = stripped.find(']')?;
    let (id, status, valid) = parse_bracket(&stripped[..close]);
    let _ = status;
    Some((id, valid))
}

fn parse_bracket(inner: &str) -> (String, Option<String>, bool) {
    let (id, status) = match inner.split_once(':') {
        Some((id, status)) => (id.to_string(), Some(status.to_string())),
        None => (inner.to_string(), None),
    };
    let valid_id = IdGenerator::is_valid(&id);
    let valid_status = status.as_deref().map_or(true, IdGenerator::is_valid_status);
    (id, status, valid_id && valid_status)
}

fn line_ranges(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            ranges.push((start, i + 1));
            start = i + 1;
        }
    }
    if start < bytes.len() {
        ranges.push((start, bytes.len()));
    }
    ranges
}

/// Read-modify-write a whole file with the chokepoint discipline: mtime
/// snapshot before read, compare before write, atomic replace, one internal
/// retry (a second race returns `Retry` for the caller to surface). A missing
/// file is created from `seed` first. `f` returns the new full text, or None
/// for a no-op. This is the door for non-marker state files (tasks.md, log.md
/// — COS_PRD.md); marker lines keep going through `inject`/`set_status`.
pub fn guarded_rewrite<F>(path: &Path, seed: &[u8], f: F) -> io::Result<WriteResult>
where
    F: Fn(&str) -> Option<String>,
{
    for _ in 0..2 {
        let before = match fs::metadata(path) {
            Ok(m) => m.modified().ok(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                atomic_write(path, seed)?;
                fs::metadata(path)?.modified().ok()
            }
            Err(e) => return Err(e),
        };
        let text = fs::read_to_string(path)?;
        let Some(new_text) = f(&text) else {
            return Ok(WriteResult::Unchanged);
        };
        let after = fs::metadata(path)?.modified().ok();
        if before != after {
            continue; // raced a concurrent writer; retry once
        }
        atomic_write(path, new_text.as_bytes())?;
        return Ok(WriteResult::Written);
    }
    Ok(WriteResult::Retry)
}

pub fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(content)?;
    tmp.as_file_mut().sync_all()?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct OwnWriteTracker {
    writes: HashMap<PathBuf, Instant>,
}

impl OwnWriteTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, path: &Path) {
        self.writes.insert(path.to_path_buf(), Instant::now());
    }

    pub fn is_suppressed(&self, path: &Path) -> bool {
        self.writes
            .get(path)
            .map(|at| at.elapsed() < Duration::from_millis(500))
            .unwrap_or(false)
    }

    pub fn cleanup(&mut self) {
        self.writes
            .retain(|_, at| at.elapsed() < Duration::from_millis(500));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn tmp() -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "indiana-write-{nanos}-{}",
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn request(path: &Path, line_no: usize) -> InjectRequest {
        InjectRequest {
            path: path.to_path_buf(),
            line_no,
        }
    }

    #[test]
    fn test_id_first_injection() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::action do thing\n").unwrap();

        let result = inject(&[request(&file, 1)], false);
        assert_eq!(result[&file], WriteResult::Written);
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.starts_with("::action["));
        assert!(text.ends_with("] do thing\n"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_id_idempotent() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::todo[frata-nimta:done] do thing\r\n").unwrap();
        let before = fs::read(&file).unwrap();

        let result = inject(&[request(&file, 1)], false);
        assert_eq!(result[&file], WriteResult::Unchanged);
        assert_eq!(before, fs::read(&file).unwrap());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_bracket_repair_bad_id() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::action[Bad-ID] do thing").unwrap();

        let result = inject(&[request(&file, 1)], false);
        assert_eq!(result[&file], WriteResult::Written);
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.starts_with("::action["));
        assert!(!text.contains("Bad-ID"));
        assert!(!text.ends_with('\n'));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_bracket_repair_bad_status() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::action[frata-nimta:unknown] do thing\n").unwrap();

        inject(&[request(&file, 1)], false);
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "::action[frata-nimta] do thing\n"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_scan_read_only_no_write() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::action do thing\n").unwrap();
        let before = fs::read(&file).unwrap();

        let result = inject(&[request(&file, 1)], true);
        assert_eq!(result[&file], WriteResult::Unchanged);
        assert_eq!(before, fs::read(&file).unwrap());
        fs::remove_dir_all(dir).ok();
    }

    // ── set_status: the auto-run claim / completion write (IN_AUTORUN.md) ──

    #[test]
    fn test_claim_working_mints_id_and_strips_flag() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "some text ::fix -a banana\n").unwrap();

        let result = set_status(&file, 1, "working").unwrap();
        assert_eq!(result, WriteResult::Written);
        let text = fs::read_to_string(&file).unwrap();
        // `-a` gone; id minted; status working; message + spacing preserved.
        assert!(text.starts_with("some text ::fix["), "got: {text:?}");
        assert!(text.contains(":working] banana\n"), "got: {text:?}");
        assert!(!text.contains("-a"), "flag not stripped: {text:?}");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_claim_working_idempotent() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::fix[happy-otter:working] banana\n").unwrap();
        let before = fs::read(&file).unwrap();

        let result = set_status(&file, 1, "working").unwrap();
        assert_eq!(result, WriteResult::Unchanged);
        assert_eq!(before, fs::read(&file).unwrap());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_claim_working_no_message() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::fix -a\n").unwrap();

        set_status(&file, 1, "working").unwrap();
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.starts_with("::fix["));
        assert!(text.ends_with(":working]\n"), "no trailing space: {text:?}");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_claim_group_retains_group_and_strips_auto_in_either_order() {
        for (i, flags) in ["-1 -a", "-a -1"].into_iter().enumerate() {
            let dir = tmp();
            let file = dir.join(format!("doc-{i}.md"));
            fs::write(&file, format!("::fix {flags} banana\n")).unwrap();

            set_status(&file, 1, "working").unwrap();
            let text = fs::read_to_string(&file).unwrap();
            assert!(text.contains(":working] -1 banana\n"), "got: {text:?}");
            assert!(!text.contains("-a"), "auto flag survived: {text:?}");
            fs::remove_dir_all(dir).ok();
        }
    }

    #[test]
    fn test_working_to_done_transition() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "::fix[happy-otter:working] banana\n").unwrap();

        let result = set_status(&file, 1, "done").unwrap();
        assert_eq!(result, WriteResult::Written);
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "::fix[happy-otter:working] banana\n".replace("working", "done")
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_working_to_failed_transition() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "line ::elaborate[lurvo-pannik:working] do it\n").unwrap();

        set_status(&file, 1, "failed").unwrap();
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "line ::elaborate[lurvo-pannik:failed] do it\n"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_claim_preserves_other_lines() {
        let dir = tmp();
        let file = dir.join("doc.md");
        fs::write(&file, "keep me\n::fix -a banana\nkeep me too\n").unwrap();

        set_status(&file, 2, "working").unwrap();
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.starts_with("keep me\n::fix["));
        assert!(text.ends_with("] banana\nkeep me too\n"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_own_write_tracker() {
        let mut tracker = OwnWriteTracker::new();
        let path = PathBuf::from("/tmp/indiana-own-write.md");
        assert!(!tracker.is_suppressed(&path));
        tracker.record(&path);
        assert!(tracker.is_suppressed(&path));
        std::thread::sleep(Duration::from_millis(550));
        assert!(!tracker.is_suppressed(&path));
    }
}
