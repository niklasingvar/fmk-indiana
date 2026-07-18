//! Auto-run dispatch orchestration (IN_AUTORUN.md). After each index rebuild
//! the daemon calls [`Dispatcher::consider`], which claims fresh `-a` markers
//! (rewriting them to `[id:working]` through the core write chokepoint) and
//! runs one ACP turn per marker on a background thread. Completion is decided
//! by re-scanning the file: if the agent removed the marker line the work is
//! resolved; if the `:working` marker is still there, it is marked `:failed`.

use crate::acp::AcpAgent;
use crate::config::Config;
use crate::paths::indiana_dir;
use indiana_core::agents::{self, AgentCatalog};
use indiana_core::compile::{
    compile_one, compile_with_options, render_agent_dispatch, render_dispatch,
    render_group_dispatch, CompileOptions, CompiledPayload,
};
use indiana_core::cos;
use indiana_core::index::{Index, Located};
use indiana_core::system_prompt::SystemPrompt;
use indiana_core::markers;
use indiana_core::parser::Status;
use indiana_core::write::{self, OwnWriteTracker, WriteResult};
use indiana_protocol::{
    AgentJob, AgentJobState, AgentQuestion, ElicitationAction, TranscriptEvent, TranscriptEventKind,
};
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

/// Ceiling on retained transcript events per job. Older events fall off the
/// front; `seq` keeps increasing, so a poller past the window just resumes
/// from what is retained.
const MAX_TRANSCRIPT_EVENTS: usize = 1000;

/// A one-shot answer that resumes the ACP request currently waiting in a turn.
struct PendingAnswer {
    action: ElicitationAction,
    answer: Option<String>,
}

struct JobEntry {
    job: AgentJob,
    answer: Option<SyncSender<PendingAnswer>>,
    /// Chat-shaped projection of the turn's `session/update` stream, plus the
    /// questions asked and answers given. Dies with the job, like the job.
    transcript: Vec<TranscriptEvent>,
    next_seq: u64,
}

/// Live agent processes are daemon memory, projected through the Unix socket
/// so faces can render them and reply to their ACP elicitation requests.
#[derive(Clone, Default)]
struct JobRegistry {
    entries: Arc<Mutex<BTreeMap<String, JobEntry>>>,
}

impl JobRegistry {
    fn insert(&self, job: AgentJob) {
        self.entries.lock().unwrap().insert(
            job.id.clone(),
            JobEntry {
                job,
                answer: None,
                transcript: Vec::new(),
                next_seq: 0,
            },
        );
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
        self.push_event(
            id,
            TranscriptEventKind::Question,
            question.message.clone(),
            false,
        );
        let response = receiver
            .recv()
            .map_err(|_| io::Error::other("agent question was interrupted"))?;
        let answer_text = match response.action {
            ElicitationAction::Accept => response.answer.clone().unwrap_or_default(),
            ElicitationAction::Decline => "declined".to_string(),
            ElicitationAction::Cancel => "cancelled".to_string(),
        };
        self.push_event(id, TranscriptEventKind::Answer, answer_text, false);
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

    /// Project one `session/update` notification into the job's transcript.
    /// Message and thought chunks merge into their predecessor so a streamed
    /// sentence is one event, not fifty.
    fn append_update(&self, id: &str, params: &Value) {
        let Some(update) = params.get("update") else {
            return;
        };
        let (kind, text, merge) = match update.get("sessionUpdate").and_then(Value::as_str) {
            Some("agent_message_chunk") => match chunk_text(update) {
                Some(text) => (TranscriptEventKind::Agent, text, true),
                None => return,
            },
            Some("agent_thought_chunk") => match chunk_text(update) {
                Some(text) => (TranscriptEventKind::Thought, text, true),
                None => return,
            },
            Some("tool_call") | Some("tool_call_update") => {
                let Some(title) = update
                    .get("title")
                    .and_then(Value::as_str)
                    .or_else(|| update.get("toolCallId").and_then(Value::as_str))
                else {
                    return;
                };
                let text = match update.get("status").and_then(Value::as_str) {
                    Some(status) => format!("{title} ({status})"),
                    None => title.to_string(),
                };
                (TranscriptEventKind::Tool, text, false)
            }
            _ => return,
        };
        self.push_event(id, kind, text, merge);
    }

    fn push_event(&self, id: &str, kind: TranscriptEventKind, text: String, merge: bool) {
        let mut entries = self.entries.lock().unwrap();
        let Some(entry) = entries.get_mut(id) else {
            return;
        };
        if merge {
            if let Some(last) = entry.transcript.last_mut() {
                if last.kind == kind {
                    last.text.push_str(&text);
                    return;
                }
            }
        }
        let seq = entry.next_seq;
        entry.next_seq += 1;
        entry.transcript.push(TranscriptEvent { seq, kind, text });
        if entry.transcript.len() > MAX_TRANSCRIPT_EVENTS {
            let excess = entry.transcript.len() - MAX_TRANSCRIPT_EVENTS;
            entry.transcript.drain(..excess);
        }
    }

    /// Events at or after `since_seq`, plus the seq to poll from next.
    /// `None` when the job is gone (turn ended).
    fn transcript(&self, id: &str, since_seq: u64) -> Option<(Vec<TranscriptEvent>, u64)> {
        let entries = self.entries.lock().unwrap();
        let entry = entries.get(id)?;
        let events = entry
            .transcript
            .iter()
            .filter(|event| event.seq >= since_seq)
            .cloned()
            .collect();
        Some((events, entry.next_seq))
    }
}

fn chunk_text(update: &Value) -> Option<String> {
    update
        .get("content")?
        .get("text")?
        .as_str()
        .map(str::to_string)
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
    /// Repos with a live agent turn. One turn per repo at a time, so
    /// concurrent agents never race on the shared working tree / git commit
    /// (IN_AUTORUN.md). Different repos still run in parallel.
    inflight_roots: Arc<Mutex<HashSet<PathBuf>>>,
    jobs: JobRegistry,
}

/// How a manual batch selects its members: a numeric group label (`-3`) or a
/// named agent persona (`-mike`). The selector also decides which system
/// prompt and coda the dispatched turn carries.
#[derive(Debug, Clone)]
enum BatchSelector {
    Group(u64),
    Agent(String),
}

impl BatchSelector {
    fn matches(&self, marker: &Located) -> bool {
        match self {
            BatchSelector::Group(group) => marker.group == Some(*group),
            BatchSelector::Agent(name) => marker.agent.as_deref() == Some(name.as_str()),
        }
    }

    /// Unique inflight-key fragment. Group labels are digits and agent names
    /// start with a letter, so the two spaces cannot collide.
    fn label(&self) -> String {
        match self {
            BatchSelector::Group(group) => group.to_string(),
            BatchSelector::Agent(name) => name.clone(),
        }
    }

    fn job_prefix(&self) -> String {
        match self {
            BatchSelector::Group(group) => format!("group-{group}"),
            BatchSelector::Agent(name) => format!("agent-{name}"),
        }
    }

    /// Human-facing detail for the chief-of-staff log.
    fn detail(&self) -> String {
        match self {
            BatchSelector::Group(group) => format!("group -{group}"),
            BatchSelector::Agent(name) => format!("agent -{name}"),
        }
    }

    fn compile_options(&self) -> CompileOptions {
        match self {
            BatchSelector::Group(group) => CompileOptions {
                group: Some(*group),
                ..Default::default()
            },
            BatchSelector::Agent(name) => CompileOptions {
                agent: Some(name.clone()),
                ..Default::default()
            },
        }
    }

    /// A group speaks with the root's default prompt; an agent with its own.
    fn system_prompt(&self, root: &Path) -> SystemPrompt {
        match self {
            BatchSelector::Group(_) => SystemPrompt::for_root(root),
            BatchSelector::Agent(name) => agents::system_prompt_for_agent(root, name),
        }
    }

    fn render(&self, payload: &CompiledPayload, system_prompt: &SystemPrompt) -> String {
        match self {
            BatchSelector::Group(group) => render_group_dispatch(payload, *group, system_prompt),
            BatchSelector::Agent(name) => render_agent_dispatch(payload, name, system_prompt),
        }
    }
}

/// RAII reservation of a repo for one agent turn: acquired before a claim,
/// released when the turn's worker (or an early-return path) drops it.
struct RootLease {
    roots: Arc<Mutex<HashSet<PathBuf>>>,
    root: PathBuf,
}

impl RootLease {
    fn acquire(roots: &Arc<Mutex<HashSet<PathBuf>>>, root: &Path) -> Option<RootLease> {
        let mut held = roots.lock().unwrap();
        if !held.insert(root.to_path_buf()) {
            return None; // repo busy — a turn is already running there
        }
        Some(RootLease {
            roots: Arc::clone(roots),
            root: root.to_path_buf(),
        })
    }
}

impl Drop for RootLease {
    fn drop(&mut self) {
        self.roots.lock().unwrap().remove(&self.root);
    }
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

    /// A live turn's transcript from `since_seq` on, for the follow view.
    pub fn job_transcript(&self, id: &str, since_seq: u64) -> Option<(Vec<TranscriptEvent>, u64)> {
        self.jobs.transcript(id, since_seq)
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
        self.run_batch(index, root, BatchSelector::Group(group), roots, config, own_writes)
    }

    /// Claim every marker tagged for one named agent persona and dispatch the
    /// compiled batch as a single ACP turn carrying the agent's own system
    /// prompt (`.indiana/agents/<name>/SYSTEM_PROMPT.md`).
    pub fn run_agent(
        &self,
        index: &Index,
        root: &Path,
        agent: &str,
        roots: &[PathBuf],
        config: &Config,
        own_writes: &Arc<Mutex<OwnWriteTracker>>,
    ) -> usize {
        if agent.trim().is_empty() {
            return 0;
        }
        self.run_batch(
            index,
            root,
            BatchSelector::Agent(agent.to_string()),
            roots,
            config,
            own_writes,
        )
    }

    /// Shared manual-batch dispatch: numeric groups and agent personas differ
    /// only in how members are selected and which system prompt leads.
    fn run_batch(
        &self,
        index: &Index,
        root: &Path,
        selector: BatchSelector,
        roots: &[PathBuf],
        config: &Config,
        own_writes: &Arc<Mutex<OwnWriteTracker>>,
    ) -> usize {
        // One turn per repo, same as `-a` dispatch: a batch run also edits and
        // commits, so it must not overlap another agent in this working tree.
        let Some(lease) = RootLease::acquire(&self.inflight_roots, root) else {
            return 0;
        };
        let key = format!("batch:{}:{}", root.display(), selector.label());
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
                    && selector.matches(marker)
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

        // Re-scan with the root's agent catalog so persona tags still resolve.
        let catalog = AgentCatalog::for_root(root);
        let mut claimed = Index::default();
        for path in &paths {
            claimed.scan_file_with(path, &catalog);
        }
        claimed.markers.retain(|marker| {
            marker.path.starts_with(root)
                && selector.matches(marker)
                && marker.status == Some(Status::Working)
        });
        if claimed.markers.is_empty() {
            self.inflight.lock().unwrap().remove(&key);
            return 0;
        }
        for marker in &claimed.markers {
            if let Some(id) = &marker.id {
                let rel = marker.path.strip_prefix(root).unwrap_or(&marker.path);
                cos_log(
                    root,
                    "claimed",
                    id,
                    &format!(
                        "{} {}:{} ({})",
                        markers::long_name(marker.kind),
                        rel.display(),
                        marker.line,
                        selector.detail()
                    ),
                    own_writes,
                );
            }
        }

        let payload = compile_with_options(
            &claimed,
            &CompileOptions {
                roots: Some(roots.to_vec()),
                ..selector.compile_options()
            },
        );
        let system_prompt = selector.system_prompt(root);
        let prompt = selector.render(&payload, &system_prompt);
        let model = agent_model(root);
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
            .unwrap_or_else(|| selector.job_prefix());
        let job_id = format!("{}-{log_id}", selector.job_prefix());
        self.jobs.insert(AgentJob {
            id: job_id.clone(),
            root: root.clone(),
            markers: paths.iter().cloned().collect(),
            state: AgentJobState::Running,
            question: None,
        });
        let jobs = self.jobs.clone();
        let this = self.clone();
        let config = config.clone();
        let roots = roots.to_vec();
        std::thread::spawn(move || {
            run_batch_turn(
                &agent,
                &root,
                &paths,
                &ids,
                &selector,
                &prompt,
                model.as_deref(),
                &own_writes,
                &jobs,
                &job_id,
            );
            jobs.remove(&job_id);
            inflight.lock().unwrap().remove(&key);
            drop(lease);
            // Chain: `-a` candidates that waited on this repo dispatch now.
            let next = Index::build_read_only(&root);
            this.consider(&next, &roots, &config, &own_writes);
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
        // One turn per repo (IN_AUTORUN.md): reserve the root *before* claiming,
        // so a busy repo's candidates stay untouched (`-a`, no status) and are
        // picked up when the running turn finishes. Concurrent agents in one
        // working tree race each other's edits and commits — never allow it.
        let root = owning_root(path, roots).unwrap_or_else(|| {
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        });
        let Some(lease) = RootLease::acquire(&self.inflight_roots, &root) else {
            return;
        };

        // Claim: mint an id and set `:working` (flags stay in source). A fresh
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

        let rel = path.strip_prefix(&root).unwrap_or(path);
        cos_log(
            &root,
            "claimed",
            &id,
            &format!("{} {}:{line}", markers::long_name(marker.kind), rel.display()),
            own_writes,
        );

        let system_prompt = SystemPrompt::for_root(&root);
        let prompt = render_dispatch(&compile_one(&marker, roots), &system_prompt);
        let model = agent_model(&root);
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
        let this = self.clone();
        let config = config.clone();
        let roots = roots.to_vec();

        std::thread::spawn(move || {
            run_turn(
                &agent,
                &root,
                &path,
                &id,
                &prompt,
                model.as_deref(),
                &own_writes,
                &jobs,
            );
            jobs.remove(&id);
            inflight.lock().unwrap().remove(&id);
            drop(lease);
            // Chain: dispatch the next candidate that waited on this repo.
            // The agent's edits usually retrigger the watcher anyway, but a
            // turn that fails without touching files would otherwise leave
            // waiting markers stranded until the next unrelated save.
            let next = Index::build_read_only(&root);
            this.consider(&next, &roots, &config, &own_writes);
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
    model: Option<&str>,
    own_writes: &Arc<Mutex<OwnWriteTracker>>,
    jobs: &JobRegistry,
) {
    let mut log = open_log(id);
    let jobs = jobs.clone();
    let mut on_question = |params: &Value| jobs.await_answer(id, params);
    let mut on_update = |params: &Value| jobs.append_update(id, params);
    let outcome = AcpAgent::spawn(agent, &mut log)
        .and_then(|mut a| a.run_turn(root, prompt, model, &mut on_question, &mut on_update));

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
            cos_log(root, "failed", id, "marker survived turn", own_writes);
        }
        (Ok(reason), None) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# resolved: marker removed by agent ({reason})\n").as_bytes(),
            );
            cos_log(root, "done", id, "", own_writes);
        }
        (Err(e), surviving) => {
            let _ =
                std::io::Write::write_all(&mut log, format!("# dispatch error: {e}\n").as_bytes());
            if let Some(m) = surviving {
                mark_failed(path, m.line, own_writes);
            }
            cos_log(root, "failed", id, &format!("dispatch error: {e}"), own_writes);
        }
    }
}

fn run_batch_turn(
    agent: &crate::config::AgentConfig,
    root: &Path,
    paths: &HashSet<PathBuf>,
    ids: &HashSet<String>,
    selector: &BatchSelector,
    prompt: &str,
    model: Option<&str>,
    own_writes: &Arc<Mutex<OwnWriteTracker>>,
    jobs: &JobRegistry,
    job_id: &str,
) {
    let detail = selector.detail();
    let mut log = open_log(job_id);
    let jobs = jobs.clone();
    let mut on_question = |params: &Value| jobs.await_answer(job_id, params);
    let mut on_update = |params: &Value| jobs.append_update(job_id, params);
    let outcome = AcpAgent::spawn(agent, &mut log)
        .and_then(|mut a| a.run_turn(root, prompt, model, &mut on_question, &mut on_update));

    let catalog = AgentCatalog::for_root(root);
    let mut fresh = Index::default();
    for path in paths {
        fresh.scan_file_with(path, &catalog);
    }
    let surviving: Vec<_> = fresh
        .markers
        .into_iter()
        .filter(|marker| {
            selector.matches(marker)
                && marker.status == Some(Status::Working)
                && marker.id.as_ref().is_some_and(|id| ids.contains(id))
        })
        .collect();

    match (&outcome, surviving.is_empty()) {
        (Ok(reason), true) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# resolved {detail}: all markers removed ({reason})\n").as_bytes(),
            );
            for id in ids {
                cos_log(root, "done", id, &detail, own_writes);
            }
        }
        (Ok(reason), false) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!(
                    "# {detail} turn ended ({reason}) but {} marker(s) survived → failed\n",
                    surviving.len()
                )
                .as_bytes(),
            );
            let survived: HashSet<&str> = surviving
                .iter()
                .filter_map(|m| m.id.as_deref())
                .collect();
            for id in ids {
                if survived.contains(id.as_str()) {
                    cos_log(root, "failed", id, "marker survived turn", own_writes);
                } else {
                    cos_log(root, "done", id, &detail, own_writes);
                }
            }
            for marker in surviving {
                mark_failed(&marker.path, marker.line, own_writes);
            }
        }
        (Err(error), _) => {
            let _ = std::io::Write::write_all(
                &mut log,
                format!("# {detail} dispatch error: {error}\n").as_bytes(),
            );
            for id in ids {
                cos_log(
                    root,
                    "failed",
                    id,
                    &format!("dispatch error: {error}"),
                    own_writes,
                );
            }
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

/// Append a dispatch-lifecycle event to the repo's chief-of-staff action log
/// (COS_PRD.md), recording the write so the daemon's watcher skips it.
fn cos_log(
    root: &Path,
    event: &str,
    id: &str,
    detail: &str,
    own_writes: &Arc<Mutex<OwnWriteTracker>>,
) {
    if cos::append_log(root, event, id, detail).is_ok() {
        own_writes.lock().unwrap().record(&cos::log_path(root));
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

/// Optional ACP model value selected for this repo's session. Missing, blank,
/// or non-string settings leave selection to the adapter.
fn agent_model(root: &Path) -> Option<String> {
    crate::casablanca::get(root, "model")
        .and_then(|value| value.as_str().map(str::trim).map(str::to_string))
        .filter(|model| !model.is_empty())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn registry_with_job(id: &str) -> JobRegistry {
        let registry = JobRegistry::default();
        registry.insert(AgentJob {
            id: id.to_string(),
            root: PathBuf::from("/tmp/repo"),
            markers: vec![PathBuf::from("/tmp/repo/doc.md")],
            state: AgentJobState::Running,
            question: None,
        });
        registry
    }

    fn message_chunk(text: &str) -> Value {
        json!({ "update": {
            "sessionUpdate": "agent_message_chunk",
            "content": { "type": "text", "text": text },
        }})
    }

    #[test]
    fn test_message_chunks_merge_into_one_event() {
        let registry = registry_with_job("happy-otter");
        registry.append_update("happy-otter", &message_chunk("Hello "));
        registry.append_update("happy-otter", &message_chunk("world"));
        let (events, next_seq) = registry.transcript("happy-otter", 0).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].text, "Hello world");
        assert_eq!(events[0].kind, TranscriptEventKind::Agent);
        assert_eq!(next_seq, 1);
    }

    #[test]
    fn test_tool_calls_break_merging_and_since_seq_filters() {
        let registry = registry_with_job("happy-otter");
        registry.append_update("happy-otter", &message_chunk("thinking"));
        registry.append_update(
            "happy-otter",
            &json!({ "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "call-1",
                "title": "Edit doc.md",
                "status": "in_progress",
            }}),
        );
        registry.append_update("happy-otter", &message_chunk("done"));
        let (events, next_seq) = registry.transcript("happy-otter", 0).unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[1].kind, TranscriptEventKind::Tool);
        assert_eq!(events[1].text, "Edit doc.md (in_progress)");
        assert_eq!(next_seq, 3);
        let (later, _) = registry.transcript("happy-otter", 2).unwrap();
        assert_eq!(later.len(), 1);
        assert_eq!(later[0].text, "done");
        assert!(registry.transcript("gone-job", 0).is_none());
    }

    #[test]
    fn test_transcript_caps_from_the_front_and_seq_keeps_increasing() {
        let registry = registry_with_job("happy-otter");
        for i in 0..(MAX_TRANSCRIPT_EVENTS + 5) {
            registry.append_update(
                "happy-otter",
                &json!({ "update": {
                    "sessionUpdate": "tool_call",
                    "title": format!("tool {i}"),
                }}),
            );
        }
        let (events, next_seq) = registry.transcript("happy-otter", 0).unwrap();
        assert_eq!(events.len(), MAX_TRANSCRIPT_EVENTS);
        assert_eq!(next_seq, (MAX_TRANSCRIPT_EVENTS + 5) as u64);
        assert_eq!(events[0].seq, 5);
        assert_eq!(events[0].text, "tool 5");
    }
}
