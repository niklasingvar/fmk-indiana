//! The long-lived daemon and its socket client (IN_DAEMON.md).
//! One daemon binds the socket and holds the index in memory; faces are
//! clients over the socket. Watch (M5) will refresh the held index; for now
//! it is the startup scan.

use crate::config::Config;
use crate::paths::{indiana_dir, socket_path};
use crate::protocol::{Request, Response};
use indiana_core::index::Index;
use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Build one index across several roots (the daemon may monitor many folders).
fn build_index(roots: &[PathBuf]) -> Index {
    let mut combined = Index::default();
    for root in roots {
        let part = Index::build(root);
        combined.markers.extend(part.markers);
        combined.warnings.extend(part.warnings);
    }
    combined
}

/// Run the daemon: bind the socket (recovering a stale one) and serve.
/// `root` overrides the monitored folders for this run when given.
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

    let roots = match root {
        Some(r) => vec![r],
        None => {
            let cfg = Config::load();
            if cfg.folders.is_empty() {
                vec![std::env::current_dir()?]
            } else {
                cfg.folders
            }
        }
    };

    let index = Arc::new(Mutex::new(build_index(&roots)));

    // Watch the roots; refresh the held index on debounced changes (M5,
    // IN_SCAN.md ~300 ms). The debouncer coalesces bursts into one rebuild.
    // `_debouncer` must stay alive for the daemon's lifetime or watching stops.
    let _debouncer = start_watch(roots.clone(), Arc::clone(&index))?;

    let listener = UnixListener::bind(&sock)?;
    eprintln!("indiana: serving {} folder(s) on {}", roots.len(), sock.display());

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let idx = Arc::clone(&index);
                // Sequential is fine: requests are tiny and rare. A slow
                // client cannot starve others meaningfully yet.
                if let Err(e) = handle(s, &idx) {
                    eprintln!("indiana: client error: {e}");
                }
            }
            Err(e) => eprintln!("indiana: accept error: {e}"),
        }
    }
    Ok(())
}

/// Start watching `roots`; on each debounced batch, rebuild the held index.
/// Returns the debouncer, which the caller must keep alive.
fn start_watch(
    roots: Vec<PathBuf>,
    index: Arc<Mutex<Index>>,
) -> io::Result<impl Sized> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer =
        new_debouncer(Duration::from_millis(300), None, tx).map_err(io::Error::other)?;
    for r in &roots {
        debouncer
            .watcher()
            .watch(r, RecursiveMode::Recursive)
            .map_err(io::Error::other)?;
        debouncer.cache().add_root(r, RecursiveMode::Recursive);
    }
    std::thread::spawn(move || {
        // One message per debounced batch — bursts coalesce to one rebuild.
        for res in rx {
            if res.is_ok() {
                let fresh = build_index(&roots);
                *index.lock().unwrap() = fresh;
            }
        }
    });
    Ok(debouncer)
}

fn handle(stream: UnixStream, index: &Mutex<Index>) -> io::Result<()> {
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
    let resp = match req {
        Request::Scan => Response { index: index.lock().unwrap().clone() },
    };
    let mut stream = stream;
    let body = serde_json::to_string(&resp).map_err(io::Error::other)?;
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
