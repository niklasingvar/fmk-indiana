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

fn atomic_write(path: &Path, content: &[u8]) -> io::Result<()> {
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
