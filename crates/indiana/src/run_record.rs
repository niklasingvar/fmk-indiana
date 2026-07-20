//! Durable per-run audit records. Every agent turn — auto-run, group, or
//! persona batch — leaves one markdown file under
//! `<root>/.indiana/chief-of-staff/runs/`, next to the action log
//! (COS_PRD.md).
//!
//! These are machine-only surfaces, structured-first: the run's facts
//! (outcome, timestamps, tokens, cost) live as YAML frontmatter parsed by
//! exactly one implementation — this module, faced by `indiana runs --json`.
//! The markdown body carries the transcript, the part written for humans.
//! Machine-written, pruned to a cap, safe to delete — history only, like
//! `log.md`.

use indiana_protocol::{TranscriptEvent, TranscriptEventKind};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Records kept per repo, newest first; older ones are pruned on write.
const MAX_RECORDS: usize = 200;

/// The structured facts of one finished turn — the record's frontmatter and
/// the `indiana runs --json` row. Field names are camelCase in both places so
/// the grammar exists once.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RunSummary {
    pub job: String,
    /// `done` or `failed`.
    pub outcome: String,
    /// Stop reason or failure note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// `YYYY-MM-DD HH:MM` (UTC), same shape as action-log stamps.
    pub started: String,
    pub ended: String,
    pub root: String,
    /// Root-relative marker files this turn was dispatched for.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub markers: Vec<String>,
    /// Per-turn token counts from the ACP prompt response, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_out: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_total: Option<u64>,
    /// Context window figures from `usage_update`, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_used: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_size: Option<u64>,
    /// Cumulative session cost from `usage_update`, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl RunSummary {
    /// `in 1234 out 567 tok` — for the action log's `run` line.
    pub fn token_summary(&self) -> Option<String> {
        if self.tokens_in.is_none() && self.tokens_out.is_none() {
            return None;
        }
        Some(format!(
            "in {} out {} tok",
            self.tokens_in.unwrap_or(0),
            self.tokens_out.unwrap_or(0)
        ))
    }

    /// `0.4321 USD` — for the action log's `run` line.
    pub fn cost_summary(&self) -> Option<String> {
        let amount = self.cost?;
        Some(format!("{amount:.4} {}", self.currency.as_deref().unwrap_or("USD")))
    }
}

/// One listed record: its filename plus the parsed summary.
#[derive(Debug, Clone, Serialize)]
pub struct RunListing {
    pub file: String,
    #[serde(flatten)]
    pub summary: RunSummary,
}

pub fn runs_dir(root: &Path) -> PathBuf {
    root.join(".indiana").join("chief-of-staff").join("runs")
}

/// Write one record. Returns the root-relative path of the new file so the
/// action log can point at it. A retried marker reuses its id, so the
/// filename is timestamped — every attempt keeps its own record.
pub fn write(root: &Path, summary: &RunSummary, events: &[TranscriptEvent]) -> io::Result<PathBuf> {
    let dir = runs_dir(root);
    std::fs::create_dir_all(&dir)?;
    let file = format!("{}-{}.md", file_stamp(), summary.job);
    std::fs::write(dir.join(&file), render(summary, events))?;
    Ok(PathBuf::from(".indiana/chief-of-staff/runs").join(file))
}

/// Drop the oldest records beyond [`MAX_RECORDS`] (timestamped filenames sort
/// by age). Returns the removed absolute paths so the daemon can suppress its
/// own watcher events.
pub fn prune(root: &Path) -> Vec<PathBuf> {
    let dir = runs_dir(root);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".md"))
        .collect();
    if names.len() <= MAX_RECORDS {
        return Vec::new();
    }
    names.sort();
    names
        .drain(..names.len() - MAX_RECORDS)
        .map(|name| dir.join(name))
        .filter(|path| std::fs::remove_file(path).is_ok())
        .collect()
}

/// Parse one record's frontmatter. `None` when the file is not a run record
/// (no fence, unparseable YAML, or no job) — such files are simply skipped.
pub fn parse_summary(text: &str) -> Option<RunSummary> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let summary: RunSummary = serde_yml::from_str(&rest[..end]).ok()?;
    (!summary.job.is_empty()).then_some(summary)
}

/// List records, newest first, capped at `limit`.
pub fn list(root: &Path, limit: usize) -> Vec<RunListing> {
    let dir = runs_dir(root);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.ends_with(".md"))
        .collect();
    names.sort();
    names
        .into_iter()
        .rev()
        .filter_map(|file| {
            let text = std::fs::read_to_string(dir.join(&file)).ok()?;
            Some(RunListing {
                summary: parse_summary(&text)?,
                file,
            })
        })
        .take(limit)
        .collect()
}

fn render(summary: &RunSummary, events: &[TranscriptEvent]) -> String {
    let mut out = String::new();
    out.push_str("---\nstatus: living\npurpose: Audit record of one agent run (machine-written).\napproval: approved\n");
    let yaml = serde_yml::to_string(summary).unwrap_or_default();
    out.push_str(&yaml);
    if !yaml.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n\n");
    out.push_str(&format!("# Run {} — {}\n\n## Transcript\n\n", summary.job, summary.outcome));
    if events.is_empty() {
        out.push_str("(no transcript events — the adapter streamed nothing)\n");
    }
    for event in events {
        out.push_str(&format!("**{}**\n{}\n\n", kind_name(event.kind), event.text.trim_end()));
    }
    out
}

fn kind_name(kind: TranscriptEventKind) -> &'static str {
    match kind {
        TranscriptEventKind::Agent => "agent",
        TranscriptEventKind::Thought => "thought",
        TranscriptEventKind::Tool => "tool",
        TranscriptEventKind::Question => "question",
        TranscriptEventKind::Answer => "answer",
    }
}

/// `YYYY-MM-DD-HHMMSS` (UTC) — second-resolution so a retried id gets a new
/// file, and lexicographic order is age order.
fn file_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    format!("{}-{h:02}{m:02}{s:02}", indiana_core::cos::today())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn summary() -> RunSummary {
        RunSummary {
            job: "su-nak".into(),
            outcome: "done".into(),
            detail: Some("end_turn".into()),
            started: "2026-07-19 21:14".into(),
            ended: "2026-07-19 21:16".into(),
            root: "/tmp/repo".into(),
            markers: vec!["doc.md".into()],
            tokens_in: Some(1234),
            tokens_out: Some(567),
            tokens_total: Some(1801),
            context_used: Some(45000),
            context_size: Some(200000),
            cost: Some(0.4321),
            currency: Some("USD".into()),
        }
    }

    fn events() -> Vec<TranscriptEvent> {
        vec![
            TranscriptEvent {
                seq: 0,
                kind: TranscriptEventKind::Agent,
                text: "working on it".into(),
            },
            TranscriptEvent {
                seq: 1,
                kind: TranscriptEventKind::Tool,
                text: "Edit doc.md (completed)".into(),
            },
        ]
    }

    #[test]
    fn test_record_round_trips_through_frontmatter() {
        let text = render(&summary(), &events());
        assert!(text.starts_with("---\n"), "record carries frontmatter");
        assert!(text.contains("# Run su-nak — done"));
        assert!(text.contains("**agent**\nworking on it"));
        assert!(text.contains("**tool**\nEdit doc.md (completed)"));
        let parsed = parse_summary(&text).expect("frontmatter parses");
        assert_eq!(parsed, summary(), "one grammar, byte-faithful both ways");
    }

    #[test]
    fn test_unknown_usage_fields_are_omitted_not_zeroed() {
        let lean = RunSummary {
            tokens_in: None,
            tokens_out: None,
            tokens_total: None,
            context_used: None,
            context_size: None,
            cost: None,
            currency: None,
            ..summary()
        };
        let text = render(&lean, &[]);
        assert!(!text.contains("tokensIn"));
        assert!(!text.contains("cost:"));
        assert!(text.contains("(no transcript events"));
        assert_eq!(parse_summary(&text).unwrap().tokens_in, None);
    }

    #[test]
    fn test_non_records_parse_to_none() {
        assert_eq!(parse_summary("no frontmatter"), None);
        assert_eq!(parse_summary("---\nstatus: draft\n---\nbody"), None); // no job
    }

    #[test]
    fn test_write_list_newest_first_and_prune() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let rel = write(root, &summary(), &events()).unwrap();
        assert!(rel.starts_with(".indiana/chief-of-staff/runs"));

        // An older record, planted with an earlier stamp.
        let older = RunSummary { job: "be-ka".into(), outcome: "failed".into(), ..summary() };
        std::fs::write(
            runs_dir(root).join("2020-01-01-000000-be-ka.md"),
            render(&older, &[]),
        )
        .unwrap();

        let listed = list(root, 10);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].summary.job, "su-nak", "newest first");
        assert_eq!(listed[1].summary.job, "be-ka");
        assert_eq!(list(root, 1).len(), 1, "limit caps the list");

        // Prune keeps the newest MAX_RECORDS; flood past the cap.
        for i in 0..MAX_RECORDS {
            std::fs::write(
                runs_dir(root).join(format!("2021-01-01-{:06}-x.md", i)),
                render(&older, &[]),
            )
            .unwrap();
        }
        let removed = prune(root);
        assert_eq!(removed.len(), 2, "the two oldest fall off");
        assert!(removed.iter().any(|p| p.ends_with("2020-01-01-000000-be-ka.md")));
        assert!(runs_dir(root).join(rel.file_name().unwrap()).exists(), "newest survives");
    }

    #[test]
    fn test_summaries_for_action_log() {
        assert_eq!(summary().token_summary().as_deref(), Some("in 1234 out 567 tok"));
        assert_eq!(summary().cost_summary().as_deref(), Some("0.4321 USD"));
        let lean = RunSummary { tokens_in: None, tokens_out: None, cost: None, ..summary() };
        assert_eq!(lean.token_summary(), None);
        assert_eq!(lean.cost_summary(), None);
    }
}
