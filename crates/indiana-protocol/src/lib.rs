//! Wire protocol types shared between daemon and menulet.
//! One source of truth — no duplicated type definitions.

use indiana_core::compile::CompiledPayload;
use indiana_core::index::Index;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Request {
    /// Return the daemon's current index of its monitored folders.
    Scan,
    /// Return the daemon's compiled payload.
    Payload,
    /// Monitor a new folder: persist it, watch it, and rescan now.
    Add { path: PathBuf },
    /// Return per-folder status — paths and marker counts (menulet face).
    Status,
    /// Stop monitoring a folder: remove from config, unwatch, rebuild index.
    Remove { path: PathBuf },
    /// Return the compiled bundle for one folder as ready-to-paste text.
    /// `kind` and `group` are optional filters.
    Copy {
        path: PathBuf,
        #[serde(default)]
        kind: Option<String>,
        #[serde(default)]
        group: Option<u64>,
    },
    /// Dispatch one numeric batch as a single manual ACP turn.
    RunGroup { path: PathBuf, group: u64 },
    /// Return live ACP turns so faces can render their state.
    Jobs,
    /// Answer the one pending user-input request for an ACP turn.
    AnswerJob {
        job_id: String,
        action: ElicitationAction,
        #[serde(default)]
        answer: Option<String>,
    },
    /// Return a live turn's transcript events at or after `since_seq`, so a
    /// face can follow the agent's work by polling.
    JobTranscript {
        job_id: String,
        #[serde(default)]
        since_seq: u64,
    },
    /// Graceful shutdown: ack, unlink the socket, exit.
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddResponse {
    /// False when the folder was already monitored.
    pub added: bool,
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PayloadResponse {
    pub payload: CompiledPayload,
}

/// A monitored folder + its live marker count. Computed by the daemon so the
/// menulet never counts (MENULET_PRD).
#[derive(Debug, Serialize, Deserialize)]
pub struct FolderInfo {
    pub path: String,
    pub count: usize,
    /// Numeric batches and their member counts, computed by the daemon.
    #[serde(default)]
    pub groups: Vec<GroupInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group: u64,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub folders: Vec<FolderInfo>,
    /// Whether a face can cleanly stop this daemon. False when the daemon is
    /// supervised (launchd `KeepAlive`), since a `Shutdown` would be restarted.
    /// Computed by the daemon so faces never reason about lifecycle themselves.
    /// `default` keeps older daemons (no field) deserializing as not-stoppable.
    #[serde(default)]
    pub stoppable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveResponse {
    pub removed: bool,
    pub index: Index,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CopyResponse {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunGroupResponse {
    /// False when the batch was empty or already has a live turn.
    pub accepted: bool,
    /// Number of markers claimed for this turn.
    pub count: usize,
}

/// The state a live ACP turn exposes to faces. Jobs are daemon memory: a
/// restart stops their child process, so a face must never treat this as a
/// durable work record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentJobState {
    Running,
    AwaitingInput,
}

/// A small chat-shaped form: one text answer for the requested schema field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentQuestion {
    pub message: String,
    pub field: String,
}

/// One live agent turn, owned by the daemon and projected to faces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentJob {
    pub id: String,
    pub root: PathBuf,
    pub markers: Vec<PathBuf>,
    pub state: AgentJobState,
    #[serde(default)]
    pub question: Option<AgentQuestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobsResponse {
    pub jobs: Vec<AgentJob>,
}

/// The three ACP outcomes for a human input request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ElicitationAction {
    Accept,
    Decline,
    Cancel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerJobResponse {
    /// False if the turn ended or no longer awaits an answer.
    pub accepted: bool,
}

/// What one transcript entry represents in the chat-shaped follow view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptEventKind {
    Agent,
    Thought,
    Tool,
    Question,
    Answer,
}

/// One entry of a live turn's transcript. `seq` is monotonic per job so a
/// face can poll with `since_seq` and only receive what it hasn't seen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptEvent {
    pub seq: u64,
    pub kind: TranscriptEventKind,
    pub text: String,
}

/// Transcripts are daemon memory like the jobs they belong to: they vanish
/// when the turn ends (`found: false`). The raw ACP log on disk remains the
/// durable record.
#[derive(Debug, Serialize, Deserialize)]
pub struct JobTranscriptResponse {
    pub found: bool,
    pub events: Vec<TranscriptEvent>,
    pub next_seq: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_requests_round_trip() {
        let copy = Request::Copy {
            path: PathBuf::from("/tmp/repo"),
            kind: None,
            group: Some(7),
        };
        let json = serde_json::to_string(&copy).unwrap();
        assert!(json.contains(r#""group":7"#));
        assert!(matches!(
            serde_json::from_str::<Request>(&json).unwrap(),
            Request::Copy { group: Some(7), .. }
        ));

        let run = Request::RunGroup {
            path: PathBuf::from("/tmp/repo"),
            group: 7,
        };
        let json = serde_json::to_string(&run).unwrap();
        assert!(matches!(
            serde_json::from_str::<Request>(&json).unwrap(),
            Request::RunGroup { group: 7, .. }
        ));
    }

    #[test]
    fn test_job_transcript_round_trip() {
        let req: Request =
            serde_json::from_str(r#"{"cmd":"jobtranscript","job_id":"happy-otter"}"#).unwrap();
        assert!(matches!(
            req,
            Request::JobTranscript { ref job_id, since_seq: 0 } if job_id == "happy-otter"
        ));

        let response = JobTranscriptResponse {
            found: true,
            events: vec![TranscriptEvent {
                seq: 3,
                kind: TranscriptEventKind::Agent,
                text: "working on it".into(),
            }],
            next_seq: 4,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""kind":"agent""#));
        let back: JobTranscriptResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(back.events, response.events);
        assert_eq!(back.next_seq, 4);
    }

    #[test]
    fn test_folder_group_counts_round_trip() {
        let info = FolderInfo {
            path: "/tmp/repo".into(),
            count: 4,
            groups: vec![
                GroupInfo { group: 1, count: 3 },
                GroupInfo { group: 2, count: 1 },
            ],
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: FolderInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.groups[0], GroupInfo { group: 1, count: 3 });
    }
}
