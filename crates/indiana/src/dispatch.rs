//! Auto-run dispatch orchestration (IN_AUTORUN.md). After each index rebuild
//! the daemon calls [`Dispatcher::consider`], which claims fresh `-a` markers
//! (rewriting them to `[id:working]` through the core write chokepoint) and
//! runs one ACP turn per marker on a background thread. Completion is decided
//! by re-scanning the file: if the agent removed the marker line the work is
//! resolved; if the `:working` marker is still there, it is marked `:failed`.

use crate::acp::AcpAgent;
use crate::config::Config;
use crate::paths::indiana_dir;
use indiana_core::compile::{
    compile_one, compile_with_options, render_dispatch, render_group_dispatch, CompileOptions,
};
use indiana_core::index::Index;
use indiana_core::markers;
use indiana_core::parser::Status;
use indiana_core::write::{self, OwnWriteTracker, WriteResult};
use indiana_protocol::{AgentJob, AgentJobState, AgentQuestion, ElicitationAction};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::{Arc, Mutex};

/// Ceiling on simultaneous agent turns. Beyond this, extra candidates wait for
/// a later rebuild cycle rather than spawning an unbounded fleet.
const MAX_INFLIGHT: usize = 3;

/// A one-shot answer that resumes the ACP request currently waiting in a turn.
struct PendingAnswer {
    action: ElicitationAction,
    answer: Option<String>,
}

struct JobEntry {
    job: AgentJob,
    answer: Option<SyncSender<PendingAnswer>>,
}

/// Live agent processes are daemon memory, projected through the Unix socket
/// so faces can render them and reply to their ACP elicitation requests.
#[derive(Clone, Default)]
struct JobRegistry {
    entries: Arc<Mutex<BTreeMap<String, JobEntry>>>,
}

impl JobRegistry {
    fn insert(&self, job: AgentJob) {
        self.entries
            .lock()
            .unwrap()
            .insert(job.id.clone(), JobEntry { job, answer: None });
    }

    fn remove(&self, id: &str) {
        self.entries.lock().unwrap().remove(id);
    }

    fn list(&self) -> Vec<AgentJob> {
        self.entries
            .lock()
            .unwrap()
            .values()
            .map(|entry| entry.job.clone())
            .collect()
    }

    fn answer(&self, id: &str, action: ElicitationAction, answer: Option<String>) -> bool {
        let sender = {
            let mut entries = self.entries.lock().unwrap();
            let Some(entry) = entries.get_mut(id) else {
                return false;
            };
            if entry.job.state != AgentJobState::AwaitingInput {
                return false;
            }
            let sender = entry.answer.take();
            if sender.is_some() {
                entry.job.state = AgentJobState::Running;
                entry.job.question = None;
            }
            sender
        };
        sender
            .map(|sender| sender.send(PendingAnswer { action, answer }).is_ok())
            .unwrap_or(false)
    }

    /// Surface a supported ACP form request to the face, then block this ACP
    /// thread until the human submits an answer. This is the intentional pause:
    /// no worker lock or daemon socket is held while waiting.
    fn await_answer(&self, id: &str, params: &Value) -> io::Result<Value> {
        let question = parse_question(params)?;
        let (sender, receiver) = sync_channel(1);
        {
            let mut entries = self.entries.lock().unwrap();
            let entry = entries
                .get_mut(id)
                .ok_or_else(|| io::Error::other("agent job ended before asking a question"))?;
            entry.job.state = AgentJobState::AwaitingInput;
            entry.job.question = Some(question.clone());
            entry.answer = Some(sender);
        }
        let response = receiver
            .recv()
            .map_err(|_| io::Error::other("agent question was interrupted"))?;
        let result = match response.action {
            ElicitationAction::Accept => {
                let mut content = serde_json::Map::new();
                content.insert(
                    question.field,
                    Value::String(response.answer.unwrap_or_default()),
                );
                json!({ "action": "accept", "content": content })
            }
            ElicitationAction::Decline => json!({ "action": "decline" }),
            ElicitationAction::Cancel => json!({ "action": "cancel" }),
        };
        Ok(result)
    }
}

/// Casablanca's first question UI is deliberately chat-small: one string field
/// only. The client advertises exactly this ACP form subset.
fn parse_question(params: &Value) -> io::Result<AgentQuestion> {
    if params.get("mode").and_then(Value::as_str) != Some("form") {
        return Err(io::Error::other("only ACP form elicitation is supported"));
    }
    let message = params
        .get("message")
        .and_then(Value::as_str)
        .ok_or_else(|| io::Error::other("elicitation/create has no message"))?;
    let properties = params
        .get("requestedSchema")
        .and_then(|schema| schema.get("properties"))
        .and_then(Value::as_object)
        .ok_or_else(|| io::Error::other("elicitation/create has no form properties"))?;
    if properties.len() != 1 {
        return Err(io::Error::other(
            "Casablanca supports one text answer per agent question",
        ));
    }
    let (field, schema) = properties.iter().next().expect("one property");
    if schema.get("type").and_then(Value::as_str) != Some("string") {
        return Err(io::Error::other(
            "Casablanca supports only string agent-question fields",
        ));
    }
    Ok(AgentQuestion {
        message: message.to_string(),
        field: field.clone(),
    })
}

/// Tracks which marker ids have a live agent turn, so a marker is dispatched
/// once even though it stays `:working` across several rebuilds.
#[derive(Clone, Default)]
pub struct Dispatcher {
    inflight: Arc<Mutex<HashSet<String>>>,
    /// Repos with a live auto-run turn. One `-a` turn per repo at a time, so
    /// concurrent agents never race on the shared working tree / git commit
    /// (IN_AUTORUN.md). Different repos still run in parallel.
    inflight_roots: Arc<Mutex<HashSet<PathBuf>>>,
    jobs: JobRegistry,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// The current processes, for a small face-level activity strip.
    pub fn jobs(&self) -> Vec<AgentJob> {
        self.jobs.list()
    }

    /// Resume a process paused on its one active agent question.
    pub fn answer_job(&self, id: &str, action: ElicitationAction, answer: Option<String>) -> bool {
        self.jobs.answer(id, action, answer)
    }

    /// Claim and dispatch auto-run markers found in a fresh index. Auto-run is
    /// opt-in per repo (IN_AUTORUN.md): a marker dispatches only when its owning
    /// repo enables it via `.indiana/casablanca/settings.json` `autoRun`, with
    /// the global `config.auto_run` as the fallback default.
    pub fn consider(
        &self,
        index: &Index,
        roots: &[PathBuf],
        config: &Config,
        own_writes: &Arc<Mutex<OwnWriteTracker>>,
    ) {
        // Candidates: a fresh `-a` marker (no status yet) or an orphaned
        // `:working` one (daemon restarted mid-turn, or a prior cycle crashed).
        // Both are re-attempted; a marker already in flight is skipped below.
        // The final filter keeps only markers whose owning repo opts in.
        let candidates: Vec<(PathBuf, usize)> = index
            .markers
            .iter()
            .filter(|m| {
                markers::is_auto_runnable(m.kind)
                    && match m.status {
                        None => m.auto,
                        Some(Status::Working) => true,
                        _ => false,
                    }
            })
            .filter(|m| auto_run_enabled(owning_root(&m.path, roots).as_deref(), config))
            .map(|m| (m.path.clone(), m.line))
            .collect();

        for (path, line) in candidates {
            if self.inflight.lock().unwrap().len() >= MAX_INFLIGHT {
                break;
            }
            self.try_dispatch(&path, line, roots, config, own_writes);
        }
    }

    /// Claim every marker in one repo-scoped numeric group and dispatch the
    /// compiled batch as a single ACP turn. This is an explicit user action, so
    /// it does not depend on the `auto_run` kill-switch used by `-a`.
    pub fn run_group(
        &self,
        index: &Index,
        root: &Path,
        group: u64,
        roots: &[PathBuf],
        config: &Config,
        own_writes: &Arc<Mutex<OwnWriteTracker>>,
    ) -> usize {
        if group == 0 {
            return 0;
        }
        let key = format!("group:{}:{group}", root.display());
        {
            let mut inflight = self.inflight.lock().unwrap();
            if inflight.contains(&key) || inflight.len() >= MAX_INFLIGHT {
                return 0;
            }
            inflight.insert(key.clone());
        }

        let candidates: Vec<(PathBuf, usize)> = index
            .markers
            .iter()
            .filter(|marker| {
                marker.path.starts_with(root)
                    && marker.group == Some(group)
                    && marker.status != Some(Status::Working)
                    && marker.status != Some(Status::Done)
            })
            .map(|marker| (marker.path.clone(), marker.line))
            .collect();
        if candidates.is_empty() {
            self.inflight.lock().unwrap().remove(&key);
            return 0;
        }

        let mut paths = HashSet::new();
        for (path, line) in &candidates {
            match write::set_status(path, *line, "working") {
                Ok(WriteResult::Written) => {
                    own_writes.lock().unwrap().record(path);
                    paths.insert(path.clone());
                }
                Ok(WriteResult::Unchanged) => {
                    paths.insert(path.clone());
                }
                _ => {}
            }
        }

        let mut claimed = Index::default();
        for path in &paths {
            claimed.scan_file(path);
        }
        claimed.markers.retain(|marker| {
            marker.path.starts_with(root)
                && marker.group == Some(group)
                && marker.status == Some(Status::Working)
        });
        if claimed.markers.is_empty() {
            self.inflight.lock().unwrap().remove(&key);
            return 0;
        }

        let payload = compile_with_options(
            &claimed,
            &CompileOptions {
                group: Some(group),
                roots: Some(roots.to_vec()),
                ..Default::default()
            },
        );
        let prompt = render_group_dispatch(&payload, group);
        let ids: HashSet<String> = claimed
            .markers
            .iter()
            .filter_map(|marker| marker.id.clone())
            .collect();
        let count = ids.len();
        if count == 0 {
            self.inflight.lock().unwrap().remove(&key);
            return 0;
        }

        let agent = config.agent.clone();
        let inflight = Arc::clone(&self.inflight);
        let own_writes = Arc::clone(own_writes);
        let root = root.to_path_buf();
        let log_id = ids
            .iter()
            .next()
            .cloned()
            .unwrap_or_else(|| format!("group-{group}"));
        let job_id = format!("group-{group}-{log_id}");
        self.jobs.insert(AgentJob {
            id: job_id.clone(),
            root: root.clone(),
            markers: paths.iter().cloned().collect(),
            state: AgentJobState::Running,
            question: None,
        });
        let jobs = self.jobs.clone();
        std::thread::spawn(move || {
            run_group_turn(
                &agent,
                &root,
                &paths,
                &ids,
                group,
                &prompt,
                &own_writes,
                &jobs,
                &job_id,
            );
            jobs.remove(&job_id);
            inflight.lock().unwrap().remove(&key);
        });
        count
    }

    fn try_dispatch(
        &self,
        path: &Path,
        line: usize,
        roots: &[PathBuf],
        config: &Config,
        own_writes: &Arc<Mutex<OwnWriteTracker>>,
    ) {
        // Claim: mint an id and set `:working`, stripping the `-a` flag. A fresh
        // marker is rewritten (record the own-write); an orphaned one is already
        // `:working` (idempotent Unchanged). A racing edit → Retry: skip.
        match write::set_status(path, line, "working") {
            Ok(WriteResult::Written) => own_writes.lock().unwrap().record(path),
            Ok(WriteResult::Unchanged) => {}
            _ => return,
        }

        // Re-scan the one file for the claimed marker's minted id + scope.
        let mut idx = Index::default();
        idx.scan_file(path);
        let Some(marker) = idx
            .markers
            .into_iter()
            .find(|m| m.line == line && m.status == Some(Status::Working))
        else {
            return;
        };
        let Some(id) = marker.id.clone() else {
            return;
        };

        {
            let mut inflight = self.inflight.lock().unwrap();
            if inflight.contains(&id) {
                return; // already running
            }
            inflight.insert(id.clone());
        }

        let prompt = render_dispatch(&compile_one(&marker, roots));
        let root = owning_root(path, roots).unwrap_or_else(|| {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });
        let agent = config.agent.clone();
        let inflight = Arc::clone(&self.inflight);
        let own_writes = Arc::clone(own_writes);
        let path = path.to_path_buf();
        self.jobs.insert(AgentJob {
            id: id.clone(),
            root: root.clone(),
            markers: vec![path.clone()],
            state: AgentJobState::Running,
            question: None,
        });
        let jobs = self.jobs.clone();

        std::thread::spawn(move || {
            run_turn(&agent, &root, &path, &id, &prompt, &own_writes, &jobs);
            jobs.remove(&id);
            inflight.lock().unwrap().remove(&id);
        });
    }
}

/// Run one ACP turn to completion, then reconcile the marker's state.
fn run_turn(
    agent: &crate::config::AgentConfig,
    root: &Path,
    path: &Path,
    id: &str,
    prompt: &str,
    own_writes: &Arc<Mutex<OwnWriteTracker>>,
    jobs: &JobRegistry,
) {
    let mut log = open_log(id);
    let jobs = jobs.clone();
    let mut on_question = |params: &Value| jobs.await_answer(id, params);
    let outcome = AcpAgent::spawn(agent, &mut log)
        .and_then(|mut a| a.run_turn(root, prompt, &mut on_question));

    // Decide by inspecting the file, not the stop reason: the agent resolves a
    // marker by deleting its line (IN_AUTORUN.md), so a `:working` marker that
    // survives the turn is a failure regardless of how the turn ended.
    let mut idx = Index::default();
    idx.scan_file(path);
    let surviving = idx
        .markers
        .into_iter()
        .find(|m| m.id.as_deref() == Some(id) && m.status == Some(Status::Working));

    match (&outcome, surviving) {
        (Ok(reason), Some(m)) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# turn ended ({reason}) but marker survived → failed\n").as_bytes(),
            );
            mark_failed(path, m.line, own_writes);
        }
        (Ok(reason), None) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# resolved: marker removed by agent ({reason})\n").as_bytes(),
            );
        }
        (Err(e), surviving) => {
            let _ =
                std::io::Write::write_all(&mut log, format!("# dispatch error: {e}\n").as_bytes());
            if let Some(m) = surviving {
                mark_failed(path, m.line, own_writes);
            }
        }
    }
}

fn run_group_turn(
    agent: &crate::config::AgentConfig,
    root: &Path,
    paths: &HashSet<PathBuf>,
    ids: &HashSet<String>,
    group: u64,
    prompt: &str,
    own_writes: &Arc<Mutex<OwnWriteTracker>>,
    jobs: &JobRegistry,
    job_id: &str,
) {
    let log_id = ids
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| format!("group-{group}"));
    let mut log = open_log(&format!("group-{group}-{log_id}"));
    let jobs = jobs.clone();
    let mut on_question = |params: &Value| jobs.await_answer(job_id, params);
    let outcome = AcpAgent::spawn(agent, &mut log)
        .and_then(|mut a| a.run_turn(root, prompt, &mut on_question));

    let mut fresh = Index::default();
    for path in paths {
        fresh.scan_file(path);
    }
    let surviving: Vec<_> = fresh
        .markers
        .into_iter()
        .filter(|marker| {
            marker.group == Some(group)
                && marker.status == Some(Status::Working)
                && marker.id.as_ref().is_some_and(|id| ids.contains(id))
        })
        .collect();

    match (&outcome, surviving.is_empty()) {
        (Ok(reason), true) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# resolved group -{group}: all markers removed ({reason})\n").as_bytes(),
            );
        }
        (Ok(reason), false) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!(
                    "# group -{group} turn ended ({reason}) but {} marker(s) survived → failed\n",
                    surviving.len()
                )
                .as_bytes(),
            );
            for marker in surviving {
                mark_failed(&marker.path, marker.line, own_writes);
            }
        }
        (Err(error), _) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# group -{group} dispatch error: {error}\n").as_bytes(),
            );
            for marker in surviving {
                mark_failed(&marker.path, marker.line, own_writes);
            }
        }
    }
}

fn mark_failed(path: &Path, line: usize, own_writes: &Arc<Mutex<OwnWriteTracker>>) {
    if let Ok(WriteResult::Written) = write::set_status(path, line, "failed") {
        own_writes.lock().unwrap().record(path);
    }
}

/// Whether auto-run is enabled for a marker's owning repo (IN_AUTORUN.md):
/// the repo's `.indiana/casablanca/settings.json` `autoRun` bool wins, falling
/// back to the global `config.auto_run` when the repo hasn't set it (or set a
/// non-bool). A marker outside every monitored root uses the global default.
fn auto_run_enabled(root: Option<&Path>, config: &Config) -> bool {
    match root.and_then(|root| crate::casablanca::get(root, "autoRun")) {
        Some(serde_json::Value::Bool(enabled)) => enabled,
        _ => config.auto_run,
    }
}

/// The deepest monitored root that contains `path` — the agent's working dir.
fn owning_root(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    roots
        .iter()
        .filter(|root| path.starts_with(root))
        .max_by_key(|root| root.components().count())
        .cloned()
}

/// Append-mode log at `~/.indiana/dispatch/<id>.log`; falls back to `/dev/null`
/// so a missing log never sinks a dispatch.
fn open_log(id: &str) -> File {
    let dir = indiana_dir().join("dispatch");
    let _ = std::fs::create_dir_all(&dir);
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("{id}.log")))
        .or_else(|_| File::create("/dev/null"))
        .expect("open dispatch log or /dev/null")
}
