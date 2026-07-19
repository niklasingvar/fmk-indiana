//! Durable per-run audit records. Every agent turn — auto-run, group, or
//! persona batch — leaves one markdown file under
//! `<root>/.indiana/chief-of-staff/runs/`, next to the action log
//! (COS_PRD.md): what ran, what the agent did (the transcript that otherwise
//! dies with the job), how it ended, and what it consumed (tokens, context,
//! cost) when the ACP adapter reports them.
//!
//! Machine-written, append-never (one file per run), safe to delete —
//! history only, like `log.md`.

use crate::dispatch::RunUsage;
use indiana_core::cos;
use indiana_protocol::{TranscriptEvent, TranscriptEventKind};
use serde_json::Value;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Everything one finished turn leaves behind.
pub struct RunRecord<'a> {
    pub job_id: &'a str,
    pub root: &'a Path,
    /// Root-relative marker files this turn was dispatched for.
    pub markers: Vec<String>,
    pub started: String,
    pub ended: String,
    /// `done` or `failed`.
    pub outcome: &'a str,
    /// Failure reason or stop-reason note; empty when there is nothing to say.
    pub detail: String,
    /// The prompt response's per-turn token counts (ACP `usage`), when sent.
    pub tokens: Option<Value>,
    /// Context window + cumulative cost from `usage_update`, when sent.
    pub usage: Option<RunUsage>,
    pub events: &'a [TranscriptEvent],
}

pub fn runs_dir(root: &Path) -> PathBuf {
    root.join(".indiana").join("chief-of-staff").join("runs")
}

/// Write the record. Returns the root-relative path of the new file so the
/// action log can point at it. A retried marker reuses its id, so the
/// filename is timestamped — every attempt keeps its own record.
pub fn write(record: &RunRecord) -> io::Result<PathBuf> {
    let dir = runs_dir(record.root);
    std::fs::create_dir_all(&dir)?;
    let file = format!("{}-{}.md", file_stamp(), record.job_id);
    std::fs::write(dir.join(&file), render(record))?;
    Ok(PathBuf::from(".indiana/chief-of-staff/runs").join(file))
}

/// `in 1234 out 567 tok` — per-turn token counts from the ACP `usage` object.
/// None when the adapter sent nothing usable.
pub fn token_summary(tokens: Option<&Value>) -> Option<String> {
    let tokens = tokens?;
    let get = |key: &str| tokens.get(key).and_then(Value::as_u64);
    let (input, output) = (get("inputTokens"), get("outputTokens"));
    if input.is_none() && output.is_none() {
        return None;
    }
    Some(format!(
        "in {} out {} tok",
        input.unwrap_or(0),
        output.unwrap_or(0)
    ))
}

/// `0.4321 USD` — cumulative session cost, when reported.
pub fn cost_summary(usage: Option<&RunUsage>) -> Option<String> {
    let usage = usage?;
    let amount = usage.cost_amount?;
    Some(format!(
        "{amount:.4} {}",
        usage.cost_currency.as_deref().unwrap_or("USD")
    ))
}

fn render(record: &RunRecord) -> String {
    let mut out = String::new();
    out.push_str("---\nstatus: living\npurpose: Audit record of one agent run (machine-written).\napproval: approved\n---\n\n");
    out.push_str(&format!("# Run {} — {}\n\n", record.job_id, record.outcome));
    out.push_str(&format!("- started: {} UTC\n", record.started));
    out.push_str(&format!("- ended: {} UTC\n", record.ended));
    out.push_str(&format!("- root: {}\n", record.root.display()));
    if !record.markers.is_empty() {
        out.push_str(&format!("- markers: {}\n", record.markers.join(", ")));
    }
    out.push_str(&format!("- outcome: {}", record.outcome));
    if !record.detail.is_empty() {
        out.push_str(&format!(" ({})", record.detail));
    }
    out.push('\n');
    if let Some(tokens) = &record.tokens {
        if let Some(summary) = token_summary(Some(tokens)) {
            out.push_str(&format!("- tokens: {summary}"));
            let get = |key: &str| tokens.get(key).and_then(Value::as_u64);
            let mut cache = Vec::new();
            if let Some(read) = get("cachedReadTokens") {
                cache.push(format!("cache read {read}"));
            }
            if let Some(write) = get("cachedWriteTokens") {
                cache.push(format!("cache write {write}"));
            }
            if let Some(thought) = get("thoughtTokens") {
                cache.push(format!("thought {thought}"));
            }
            if !cache.is_empty() {
                out.push_str(&format!(" ({})", cache.join(", ")));
            }
            out.push('\n');
        }
    }
    if let Some(usage) = &record.usage {
        if let (Some(used), Some(size)) = (usage.context_used, usage.context_size) {
            out.push_str(&format!("- context: {used} of {size} tokens used\n"));
        }
        if let Some(cost) = cost_summary(Some(usage)) {
            out.push_str(&format!("- cost: {cost}\n"));
        }
    }
    out.push_str("\n## Transcript\n\n");
    if record.events.is_empty() {
        out.push_str("(no transcript events — the adapter streamed nothing)\n");
    }
    for event in record.events {
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
/// file. Date comes from cos so the two stamps can never disagree on the day.
fn file_stamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (h, m, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    format!("{}-{h:02}{m:02}{s:02}", cos::today())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record<'a>(root: &'a Path, events: &'a [TranscriptEvent]) -> RunRecord<'a> {
        RunRecord {
            job_id: "su-nak",
            root,
            markers: vec!["doc.md".into()],
            started: "2026-07-19 21:14".into(),
            ended: "2026-07-19 21:16".into(),
            outcome: "done",
            detail: "end_turn".into(),
            tokens: Some(json!({
                "inputTokens": 1234,
                "outputTokens": 567,
                "cachedReadTokens": 8900,
            })),
            usage: Some(RunUsage {
                context_used: Some(45000),
                context_size: Some(200000),
                cost_amount: Some(0.4321),
                cost_currency: Some("USD".into()),
            }),
            events,
        }
    }

    #[test]
    fn test_render_carries_outcome_usage_and_transcript() {
        let events = vec![
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
        ];
        let text = render(&record(Path::new("/tmp/repo"), &events));
        assert!(text.starts_with("---\n"), "record carries frontmatter");
        assert!(text.contains("# Run su-nak — done"));
        assert!(text.contains("- outcome: done (end_turn)"));
        assert!(text.contains("- tokens: in 1234 out 567 tok (cache read 8900)"));
        assert!(text.contains("- context: 45000 of 200000 tokens used"));
        assert!(text.contains("- cost: 0.4321 USD"));
        assert!(text.contains("**agent**\nworking on it"));
        assert!(text.contains("**tool**\nEdit doc.md (completed)"));
    }

    #[test]
    fn test_render_omits_unknown_usage() {
        let events: Vec<TranscriptEvent> = Vec::new();
        let mut r = record(Path::new("/tmp/repo"), &events);
        r.tokens = None;
        r.usage = None;
        let text = render(&r);
        assert!(!text.contains("- tokens:"));
        assert!(!text.contains("- cost:"));
        assert!(text.contains("(no transcript events"));
    }

    #[test]
    fn test_write_lands_under_runs_and_returns_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        let events: Vec<TranscriptEvent> = Vec::new();
        let rel = write(&record(dir.path(), &events)).unwrap();
        assert!(rel.starts_with(".indiana/chief-of-staff/runs"));
        let text = std::fs::read_to_string(dir.path().join(&rel)).unwrap();
        assert!(text.contains("# Run su-nak — done"));
        let name = rel.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.ends_with("-su-nak.md"), "timestamped filename: {name}");
    }

    #[test]
    fn test_token_summary_none_without_counts() {
        assert_eq!(token_summary(Some(&json!({}))), None);
        assert_eq!(token_summary(None), None);
        assert_eq!(
            token_summary(Some(&json!({ "inputTokens": 5 }))),
            Some("in 5 out 0 tok".into())
        );
    }
}
