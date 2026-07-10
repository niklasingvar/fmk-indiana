//! Auto-run dispatch orchestration (IN_AUTORUN.md). After each index rebuild
//! the daemon calls [`Dispatcher::consider`], which claims fresh `-a` markers
//! (rewriting them to `[id:working]` through the core write chokepoint) and
//! runs one ACP turn per marker on a background thread. Completion is decided
//! by re-scanning the file: if the agent removed the marker line the work is
//! resolved; if the `:working` marker is still there, it is marked `:failed`.

use crate::acp::AcpAgent;
use crate::config::Config;
use crate::paths::indiana_dir;
use indiana_core::compile::{compile_one, render_dispatch};
use indiana_core::index::Index;
use indiana_core::markers;
use indiana_core::parser::Status;
use indiana_core::write::{self, OwnWriteTracker, WriteResult};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Ceiling on simultaneous agent turns. Beyond this, extra candidates wait for
/// a later rebuild cycle rather than spawning an unbounded fleet.
const MAX_INFLIGHT: usize = 3;

/// Tracks which marker ids have a live agent turn, so a marker is dispatched
/// once even though it stays `:working` across several rebuilds.
#[derive(Clone, Default)]
pub struct Dispatcher {
    inflight: Arc<Mutex<HashSet<String>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Claim and dispatch auto-run markers found in a fresh index. Cheap and a
    /// no-op unless `config.auto_run` is on (the pausable kill-switch).
    pub fn consider(
        &self,
        index: &Index,
        roots: &[PathBuf],
        config: &Config,
        own_writes: &Arc<Mutex<OwnWriteTracker>>,
    ) {
        if !config.auto_run {
            return;
        }
        // Candidates: a fresh `-a` marker (no status yet) or an orphaned
        // `:working` one (daemon restarted mid-turn, or a prior cycle crashed).
        // Both are re-attempted; a marker already in flight is skipped below.
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
            .map(|m| (m.path.clone(), m.line))
            .collect();

        for (path, line) in candidates {
            if self.inflight.lock().unwrap().len() >= MAX_INFLIGHT {
                break;
            }
            self.try_dispatch(&path, line, roots, config, own_writes);
        }
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

        std::thread::spawn(move || {
            run_turn(&agent, &root, &path, &id, &prompt, &own_writes);
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
) {
    let mut log = open_log(id);
    let outcome = AcpAgent::spawn(agent, &mut log).and_then(|mut a| a.run_turn(root, prompt));

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

fn mark_failed(path: &Path, line: usize, own_writes: &Arc<Mutex<OwnWriteTracker>>) {
    if let Ok(WriteResult::Written) = write::set_status(path, line, "failed") {
        own_writes.lock().unwrap().record(path);
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
