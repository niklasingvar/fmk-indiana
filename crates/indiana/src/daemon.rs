//! The long-lived daemon and its socket client (IN_DAEMON.md).
//! One daemon binds the socket and holds the index in memory; faces are
//! clients over the socket. It starts monitoring the configured folders (none
//! by default) and accepts live `add` commands that watch a new folder and
//! rebuild the index immediately.

use crate::config::Config;
use crate::dispatch::Dispatcher;
use crate::paths::{indiana_dir, socket_path};
use crate::protocol::{
    AddResponse, AnswerJobResponse, CopyResponse, FolderInfo, GroupInfo, JobsResponse,
    PayloadResponse, RemoveResponse, Request, Response, RunGroupResponse, StatusResponse,
};
use indiana_core::compile::{
    compile_with_options, render_text, system_prompt_for_roots, CompileOptions, CompiledPayload,
};
use indiana_core::index::{Index, ScanReport};
use indiana_core::markers::parse_kind;
use indiana_core::templates::init_folder_indiana;
use indiana_core::write::OwnWriteTracker;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::collections::BTreeMap;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The concrete debouncer `new_debouncer` hands back.
type Deb = Debouncer<RecommendedWatcher, FileIdMap>;

/// Build one index across several roots (the daemon may monitor many folders).
fn build_index(roots: &[PathBuf]) -> ScanReport {
    let mut combined = Index::default();
    let mut written_paths = Vec::new();
    for root in roots {
        let report = Index::build_with_options(root, indiana_core::index::ScanOptions::write_ids());
        combined.markers.extend(report.index.markers);
        combined.warnings.extend(report.index.warnings);
        written_paths.extend(report.written_paths);
    }
    ScanReport {
        index: combined,
        written_paths,
    }
}

/// Watch one root recursively, keeping the debouncer's cache in sync.
fn watch_root(deb: &mut Deb, root: &Path) -> io::Result<()> {
    deb.watcher()
        .watch(root, RecursiveMode::Recursive)
        .map_err(io::Error::other)?;
    deb.cache().add_root(root, RecursiveMode::Recursive);
    Ok(())
}

/// Resolve a face-supplied path to the spelling held by the daemon index.
/// macOS may canonicalize `/var/...` to `/private/var/...`; comparing only the
/// canonical request against non-canonical configured roots makes a valid
/// folder appear empty.
fn indexed_root(path: &Path, roots: &[PathBuf]) -> PathBuf {
    if let Some(root) = roots.iter().find(|root| root.as_path() == path) {
        return root.clone();
    }
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    roots
        .iter()
        .find(|root| {
            root.canonicalize()
                .map(|candidate| candidate == canonical)
                .unwrap_or(false)
        })
        .cloned()
        .unwrap_or(canonical)
}

/// Run the daemon: bind the socket (recovering a stale one) and serve.
/// `root` overrides the monitored folders for this run when given; otherwise
/// the daemon monitors the configured folders, which may be none until an
/// `add` command selects one (IN_DAEMON.md).
pub fn serve(root: Option<PathBuf>) -> io::Result<()> {
    std::fs::create_dir_all(indiana_dir())?;
    let sock = socket_path();

    // Stale-socket recovery (IN_DAEMON.md): connect first.
    match UnixStream::connect(&sock) {
        Ok(_) => {
            eprintln!("indiana: already running at {}", sock.display());
            std::process::exit(1);
        }
        // Refused / not-a-socket / absent → no live daemon; clear and bind.
        Err(_) => {
            let _ = std::fs::remove_file(&sock);
        }
    }

    // Empty config → monitor nothing until a folder is added (IN_DAEMON.md).
    let initial_roots = match root {
        Some(r) => vec![r],
        None => Config::load().folders,
    };
    let initial_roots: Vec<PathBuf> = initial_roots
        .into_iter()
        .filter(|r| {
            if !r.is_dir() {
                eprintln!("indiana: skipping {} (not a directory)", r.display());
                false
            } else {
                true
            }
        })
        .collect();
    for root in &initial_roots {
        // A read-only or otherwise un-writable folder must not take the whole
        // daemon down — scanning it still works. Log and carry on.
        if let Err(e) = init_folder_indiana(root) {
            eprintln!(
                "indiana: skipping template init for {} ({e})",
                root.display()
            );
        }
    }

    let initial = build_index(&initial_roots);
    let roots = Arc::new(Mutex::new(initial_roots.clone()));
    let index = Arc::new(Mutex::new(initial.index));
    let own_writes = Arc::new(Mutex::new(OwnWriteTracker::new()));
    for path in initial.written_paths {
        own_writes.lock().unwrap().record(&path);
    }
    let dispatcher = Dispatcher::new();
    // Startup dispatch: pick up any `-a` markers already present, and re-attempt
    // `:working` markers orphaned by a prior crash (IN_AUTORUN.md).
    dispatcher.consider(
        &index.lock().unwrap(),
        &initial_roots,
        &Config::load(),
        &own_writes,
    );

    // Watch the roots; refresh the held index on debounced changes (M5,
    // IN_SCAN.md ~300 ms). The debouncer coalesces bursts into one rebuild and
    // must stay alive for the daemon's lifetime, so it lives in this scope and
    // the accept loop borrows it to add folders on the fly.
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer =
        new_debouncer(Duration::from_millis(300), None, tx).map_err(io::Error::other)?;
    for r in &initial_roots {
        watch_root(&mut debouncer, r)?;
    }
    spawn_watch_thread(
        rx,
        Arc::clone(&roots),
        Arc::clone(&index),
        Arc::clone(&own_writes),
        dispatcher.clone(),
    );

    let listener = UnixListener::bind(&sock)?;
    eprintln!(
        "indiana: serving {} folder(s) on {}",
        roots.lock().unwrap().len(),
        sock.display()
    );

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                // Sequential is fine: requests are tiny and rare. A slow
                // client cannot starve others meaningfully yet.
                if let Err(e) = handle(s, &roots, &index, &own_writes, &dispatcher, &mut debouncer)
                {
                    eprintln!("indiana: client error: {e}");
                }
            }
            Err(e) => eprintln!("indiana: accept error: {e}"),
        }
    }
    Ok(())
}

/// Watch thread: on each debounced batch, rebuild the held index from the
/// current roots snapshot. Reading roots fresh each time lets `add` grow the
/// monitored set without restarting the thread.
fn spawn_watch_thread(
    rx: Receiver<DebounceEventResult>,
    roots: Arc<Mutex<Vec<PathBuf>>>,
    index: Arc<Mutex<Index>>,
    own_writes: Arc<Mutex<OwnWriteTracker>>,
    dispatcher: Dispatcher,
) {
    std::thread::spawn(move || {
        // One message per debounced batch — bursts coalesce to one rebuild.
        for res in rx {
            if let Ok(events) = res {
                let paths: Vec<PathBuf> = events
                    .iter()
                    .flat_map(|event| event.paths.iter().cloned())
                    .collect();
                {
                    let mut tracker = own_writes.lock().unwrap();
                    tracker.cleanup();
                    if !paths.is_empty() && paths.iter().all(|path| tracker.is_suppressed(path)) {
                        continue;
                    }
                }

                let snapshot = roots.lock().unwrap().clone();
                let fresh = build_index(&snapshot);
                {
                    let mut tracker = own_writes.lock().unwrap();
                    for path in &fresh.written_paths {
                        tracker.record(path);
                    }
                }
                *index.lock().unwrap() = fresh.index.clone();
                // Auto-run: claim and dispatch any `-a` markers this rebuild
                // surfaced (IN_AUTORUN.md). No-op unless config.auto_run is on;
                // config is reloaded so the switch takes effect without restart.
                dispatcher.consider(&fresh.index, &snapshot, &Config::load(), &own_writes);
            }
        }
    });
}

/// Add a folder while the daemon runs: persist config, watch it, rebuild the
/// index now, and report whether it was newly added.
fn add_folder_live(
    path: &Path,
    roots: &Mutex<Vec<PathBuf>>,
    index: &Mutex<Index>,
    own_writes: &Mutex<OwnWriteTracker>,
    debouncer: &mut Deb,
) -> io::Result<AddResponse> {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut cfg = Config::load();
    let added = cfg.add_folder(&abs);
    if added {
        init_folder_indiana(&abs)?;
        cfg.save()?;
        roots.lock().unwrap().push(abs.clone());
        watch_root(debouncer, &abs)?;
        let snapshot = roots.lock().unwrap().clone();
        let fresh = build_index(&snapshot);
        {
            let mut tracker = own_writes.lock().unwrap();
            for path in &fresh.written_paths {
                tracker.record(path);
            }
        }
        *index.lock().unwrap() = fresh.index;
    }
    Ok(AddResponse {
        added,
        index: index.lock().unwrap().clone(),
    })
}

/// Stop monitoring a folder while the daemon runs: persist config, unwatch,
/// rebuild index, and report whether it was present.
fn remove_folder_live(
    path: &Path,
    roots: &Mutex<Vec<PathBuf>>,
    index: &Mutex<Index>,
    own_writes: &Mutex<OwnWriteTracker>,
    debouncer: &mut Deb,
) -> io::Result<RemoveResponse> {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut cfg = Config::load();
    let removed = cfg.remove_folder(&abs);
    if removed {
        cfg.save()?;
        {
            let mut r = roots.lock().unwrap();
            r.retain(|p| p != &abs);
        }
        // Unwatch the root: inverse of watch_root.
        let _ = debouncer.watcher().unwatch(&abs);
        debouncer.cache().remove_root(&abs);
        let snapshot = roots.lock().unwrap().clone();
        let fresh = build_index(&snapshot);
        {
            let mut tracker = own_writes.lock().unwrap();
            for path in &fresh.written_paths {
                tracker.record(path);
            }
        }
        *index.lock().unwrap() = fresh.index;
    }
    Ok(RemoveResponse {
        removed,
        index: index.lock().unwrap().clone(),
    })
}
fn handle(
    stream: UnixStream,
    roots: &Mutex<Vec<PathBuf>>,
    index: &Mutex<Index>,
    own_writes: &Arc<Mutex<OwnWriteTracker>>,
    dispatcher: &Dispatcher,
    debouncer: &mut Deb,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(()); // client hung up
    }
    let req: Request = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(e) => {
            return Err(io::Error::new(ErrorKind::InvalidData, e));
        }
    };
    let body = match req {
        Request::Scan => {
            let resp = Response {
                index: index.lock().unwrap().clone(),
            };
            serde_json::to_string(&resp).map_err(io::Error::other)?
        }
        Request::Payload => {
            let idx = index.lock().unwrap().clone();
            let snap = roots.lock().unwrap().clone();
            let resp = PayloadResponse {
                payload: compile_with_options(
                    &idx,
                    &CompileOptions {
                        roots: Some(snap),
                        ..Default::default()
                    },
                ),
            };
            serde_json::to_string(&resp).map_err(io::Error::other)?
        }
        Request::Add { path } => {
            let resp = add_folder_live(&path, roots, index, own_writes, debouncer)?;
            serde_json::to_string(&resp).map_err(io::Error::other)?
        }
        Request::Status => {
            let snap = roots.lock().unwrap().clone();
            let idx = index.lock().unwrap().clone();
            let folders: Vec<FolderInfo> = snap
                .iter()
                .map(|r| {
                    let count = idx.markers.iter().filter(|m| m.path.starts_with(r)).count();
                    let mut grouped = BTreeMap::<u64, usize>::new();
                    for marker in idx.markers.iter().filter(|m| m.path.starts_with(r)) {
                        if let Some(group) = marker.group {
                            *grouped.entry(group).or_default() += 1;
                        }
                    }
                    FolderInfo {
                        path: r.display().to_string(),
                        count,
                        groups: grouped
                            .into_iter()
                            .map(|(group, count)| GroupInfo { group, count })
                            .collect(),
                    }
                })
                .collect();
            // A launchd-supervised daemon restarts after Shutdown, so a face
            // cannot cleanly stop it (IN_DAEMON.md, MENULET_RUNBOOK.md).
            let stoppable = !crate::service::is_installed();
            serde_json::to_string(&StatusResponse { folders, stoppable })
                .map_err(io::Error::other)?
        }
        Request::Remove { path } => {
            let resp = remove_folder_live(&path, roots, index, own_writes, debouncer)?;
            serde_json::to_string(&resp).map_err(io::Error::other)?
        }
        Request::Copy { path, kind, group } => {
            let idx = index.lock().unwrap().clone();
            let snap = roots.lock().unwrap().clone();
            let abs = indexed_root(&path, &snap);
            let filtered: Vec<_> = idx
                .markers
                .iter()
                .filter(|m| m.path.starts_with(abs.as_path()))
                .cloned()
                .collect();
            let sub = Index {
                markers: filtered,
                warnings: vec![],
            };
            let kind_filter = kind.as_deref().and_then(parse_kind);
            // A provided-but-unparseable kind must not fall through to "copy
            // everything" — return an empty bundle instead of a silent copy-all.
            let text = if kind.is_some() && kind_filter.is_none() {
                String::new()
            } else {
                let opts = CompileOptions {
                    kind: kind_filter,
                    group,
                    roots: Some(vec![abs.clone()]),
                    ..Default::default()
                };
                let system_prompt = system_prompt_for_roots(std::slice::from_ref(&abs));
                render_text(&compile_with_options(&sub, &opts), &system_prompt)
            };
            serde_json::to_string(&CopyResponse { text }).map_err(io::Error::other)?
        }
        Request::RunGroup { path, group } => {
            let idx = index.lock().unwrap().clone();
            let snap = roots.lock().unwrap().clone();
            let abs = indexed_root(&path, &snap);
            let count = dispatcher.run_group(&idx, &abs, group, &snap, &Config::load(), own_writes);
            serde_json::to_string(&RunGroupResponse {
                accepted: count > 0,
                count,
            })
            .map_err(io::Error::other)?
        }
        Request::Jobs => serde_json::to_string(&JobsResponse {
            jobs: dispatcher.jobs(),
        })
        .map_err(io::Error::other)?,
        Request::AnswerJob {
            job_id,
            action,
            answer,
        } => serde_json::to_string(&AnswerJobResponse {
            accepted: dispatcher.answer_job(&job_id, action, answer),
        })
        .map_err(io::Error::other)?,
        Request::Shutdown => {
            let ack = r#"{"ok":true}"#;
            let mut stream = stream;
            stream.write_all(ack.as_bytes())?;
            stream.write_all(b"\n")?;
            stream.flush()?;
            let _ = std::fs::remove_file(socket_path());
            std::process::exit(0);
        }
    };
    let mut stream = stream;
    stream.write_all(body.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.flush()
}

/// Ask a running daemon for its index. `None` if no daemon answers — the
/// caller falls back to a standalone scan.
pub fn client_scan() -> Option<Index> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Scan).ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let resp: Response = serde_json::from_str(line.trim()).ok()?;
    Some(resp.index)
}

pub fn client_payload() -> Option<CompiledPayload> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Payload).ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let resp: PayloadResponse = serde_json::from_str(line.trim()).ok()?;
    Some(resp.payload)
}

/// Tell a running daemon to monitor `path`. `None` if no daemon answers — the
/// caller falls back to persisting config for the next `serve`.
pub fn client_add(path: &Path) -> Option<AddResponse> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Add {
        path: path.to_path_buf(),
    })
    .ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let resp: AddResponse = serde_json::from_str(line.trim()).ok()?;
    Some(resp)
}

/// Ask a running daemon for its per-folder status. `None` if no daemon answers.
pub fn client_status() -> Option<StatusResponse> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Status).ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

/// Tell a running daemon to stop monitoring `path`. `None` if no daemon answers.
pub fn client_remove(path: &Path) -> Option<RemoveResponse> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Remove {
        path: path.to_path_buf(),
    })
    .ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

/// Ask a running daemon for the copy bundle of `path`. `None` if no daemon answers.
#[allow(dead_code)]
pub fn client_copy(path: &Path) -> Option<CopyResponse> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Copy {
        path: path.to_path_buf(),
        kind: None,
        group: None,
    })
    .ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

/// Dispatch one numeric group through a running daemon.
#[allow(dead_code)]
pub fn client_run_group(path: &Path, group: u64) -> Option<RunGroupResponse> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::RunGroup {
        path: path.to_path_buf(),
        group,
    })
    .ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    serde_json::from_str(line.trim()).ok()
}

/// Tell a running daemon to shut down. Returns true if the request was sent.
#[allow(dead_code)]
pub fn client_shutdown() -> Option<()> {
    let stream = UnixStream::connect(socket_path()).ok()?;
    let mut writer = stream.try_clone().ok()?;
    let req = serde_json::to_string(&Request::Shutdown).ok()?;
    writer.write_all(req.as_bytes()).ok()?;
    writer.write_all(b"\n").ok()?;
    writer.flush().ok()?;

    // Read the ack (optional — daemon may exit before we read).
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let _ = reader.read_line(&mut line);
    Some(())
}
