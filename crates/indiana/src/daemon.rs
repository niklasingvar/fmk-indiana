//! The long-lived daemon and its socket client (IN_DAEMON.md).
//! One daemon binds the socket and holds the index in memory; faces are
//! clients over the socket. It starts monitoring the configured folders (none
//! by default) and accepts live `add` commands that watch a new folder and
//! rebuild the index immediately.

use crate::config::Config;
use crate::paths::{indiana_dir, socket_path};
use crate::protocol::{AddResponse, PayloadResponse, Request, Response};
use indiana_core::compile::{compile, CompiledPayload};
use indiana_core::index::{Index, ScanReport};
use indiana_core::write::OwnWriteTracker;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
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

    let initial = build_index(&initial_roots);
    let roots = Arc::new(Mutex::new(initial_roots.clone()));
    let index = Arc::new(Mutex::new(initial.index));
    let own_writes = Arc::new(Mutex::new(OwnWriteTracker::new()));
    for path in initial.written_paths {
        own_writes.lock().unwrap().record(&path);
    }

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
                if let Err(e) = handle(s, &roots, &index, &own_writes, &mut debouncer) {
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
                *index.lock().unwrap() = fresh.index;
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

fn handle(
    stream: UnixStream,
    roots: &Mutex<Vec<PathBuf>>,
    index: &Mutex<Index>,
    own_writes: &Mutex<OwnWriteTracker>,
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
            let resp = PayloadResponse {
                payload: compile(&idx),
            };
            serde_json::to_string(&resp).map_err(io::Error::other)?
        }
        Request::Add { path } => {
            let resp = add_folder_live(&path, roots, index, own_writes, debouncer)?;
            serde_json::to_string(&resp).map_err(io::Error::other)?
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
